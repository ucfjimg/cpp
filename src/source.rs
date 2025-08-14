//
// Wrapper for iterating over the source code, as a stream of
// characters with source location attached.
//
use crate::ccerror::CcError;
use std::path::PathBuf;

/// A location in the source code, for errors.
/// 
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    /// The index of the source file the token came from.
    pub file: u32,
    
    /// The 1-based line number in the source.
    pub line: u32,

    /// The 1-based column in the source.
    pub col: u32,
}

/// The source code from one file.
/// 
pub struct SourceFile {
    /// The name of this source file.
    pub name: PathBuf,

    /// The name, converted to a string.
    pub strname: String,

    /// The contents of the source file.
    pub text: Vec<char>,
}

/// A pointer for iterating through a source file.
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourcePointer {
    /// Index of the file being iterated.
    pub file: u32,

    /// Index of the next character.
    pub next: usize,

    /// Point in the original file.
    pub next_loc: Point,
}

/// A character from a source file.
/// 
#[derive(Debug)]
pub struct SourceChar {
    /// The character.
    pub ch: char,

    /// Its original position in the file.
    pub pt: Point,

    /// Is this the first character after a file switch?
    pub switched: bool,
}

/// The state for reading characters across all source files.
/// 
pub struct Source {
    /// All files, indexed by a file integer.
    pub files: Vec<SourceFile>,

    /// Nested stack of file pointers. The first will be the main
    /// source file.
    pub iters: Vec<SourcePointer>,

    /// The current file changed, but a character has not been read
    /// from it yet.
    pub switched: bool, 
}

/// An iterator to take source characters while a predicate is true. Unlike
/// the built-in take_while, the character past the iterated range is not
/// consumed.
/// 
pub struct TakeWhile<'a, F> 
    where F: Fn(char) -> bool
{
    /// The condition for iterating.
    pred: F, 

    /// The source code.
    source: &'a mut Source,
}

impl<F> Iterator for TakeWhile<'_, F> 
    where F: Fn(char) -> bool
{
    type Item = SourceChar;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.peek() {
            Some(sch) => {
                if (self.pred)(sch.ch) {
                    self.source.next()
                } else {
                    //
                    // Character did not pass predicate
                    //
                    None
                }
            },
            
            //
            // End of inner iterator
            //
            None => None,
        }
    }
}

impl Source {
    pub fn new() -> Source {
        Source {
            files: Vec::new(),
            iters: Vec::new(),
            switched: false,
        }
    }

    pub fn push_file(&mut self, name: &PathBuf) -> Result<(), CcError> {
        //
        // Did we already read this file?
        //
        match self.files.iter().enumerate().find(|(_, sf)| sf.name == *name) {
            Some((file, _)) => {
                let ptr = SourcePointer {
                    file: file as u32,
                    next: 0,
                    next_loc: Point {
                        file: file as u32,
                        line: 1, 
                        col: 1
                    }
                };

                self.iters.push(ptr);
                self.switched = true;
                return Ok(())
            },
            None => {},
        };

        //
        // No, read a new file.
        //
        let text = std::fs::read_to_string(name)?;
        let text = text.chars().collect();
        let file = self.files.len() as u32;

        self.files.push(SourceFile{ 
            name: name.clone(),
            strname: name.to_string_lossy().to_string(), 
            text });

        let ptr = SourcePointer {
            file: file as u32,
            next: 0,
            next_loc: Point {
                file: file as u32,
                line: 1, 
                col: 1
            }
        };

        self.iters.push(ptr);
        self.switched = true;

        self.pop_nested();

        Ok(())
    } 

    pub fn push_data(&mut self, name: &PathBuf, text: Vec<char>) {
        let file = self.files.len() as u32;

        self.files.push(SourceFile{ 
            name: name.clone(),
            strname: name.to_string_lossy().to_string(), 
            text });

        let ptr = SourcePointer {
            file: file as u32,
            next: 0,
            next_loc: Point {
                file: file as u32,
                line: 1, 
                col: 1
            }
        };

        self.iters.push(ptr);
        self.switched = true;

        self.pop_nested();
    }

    fn pop_nested(&mut self) {
        loop {
            match self.iters.last() {
                Some(sp) => {
                    if sp.next < self.files[sp.file as usize].text.len() {
                        break;
                    }
                    self.switched = true;
                    self.iters.pop();
                },
                None => break,
            }
        }
    }

    /// Get a printable name for a file, by file index.
    /// 
    pub fn get_filename(&self, file: u32) -> Option<String> {
        if (file as usize) < self.files.len() {
            Some(self.files[file as usize].strname.clone())
        } else {
            None
        }
    }

    /// Peek the next character, if there is one.
    /// 
    pub fn peek(&self) -> Option<SourceChar> {
        if self.iters.is_empty() {
            None 
        } else {
            let sp = self.iters.last().unwrap();
            let file = &self.files[sp.file as usize];
            assert!(sp.next < file.text.len());

            let ch = file.text[sp.next as usize];
            let ch = if ch == '\r' { '\n' } else { ch };
            let pt = sp.next_loc;

            Some(SourceChar{ ch, pt, switched: self.switched })
        }
    }

    /// Peek the n'th character. peek_n(0) returns the next character.
    /// 
    pub fn peek_n(&self, mut n: u32) -> Option<SourceChar> {
        //
        // We clone the iters array for skipping `n` characters. We don't need or
        // want to clone the source itself as that's a lot bigger and doesn't change.        
        //
        let mut iters = self.iters.clone();
        let mut switched = self.switched;

        if n > 0 {
            loop {
                switched = false;

                let sp = iters.last().unwrap();
                let file = &self.files[sp.file as usize];
                
                let (sp, _ch) = Source::extract_one_char(file, sp);

                *iters.last_mut().unwrap() = sp;
            
                loop {
                    match iters.last() {
                        Some(sp) => {
                            if sp.next < self.files[sp.file as usize].text.len() {
                                break;
                            }
                            iters.pop();
                            switched = true;
                        },
                        None => return None,
                    }
                }
        
                n -= 1;

                if n == 0 {
                    break;
                }
            }
        }

        match iters.last() {
            Some(sp) => {
                let file = &self.files[sp.file as usize];
                assert!(sp.next < file.text.len());
        
                let ch = file.text[sp.next as usize];
                let ch = if ch == '\r' { '\n' } else { ch };
                let pt = sp.next_loc;
        
                Some(SourceChar{ ch, pt, switched })
            },            
            None => None,
        }
    }

    fn extract_one_char(file: &SourceFile, iter: &SourcePointer) -> (SourcePointer, SourceChar) {
        let mut sp = *iter;

        assert!(iter.next < file.text.len());

        //
        // Handle CR, LF, CR/LF, LF/CR. The next layer depends on just
        // having \n to compute line splicing.         
        // 
        let ch = file.text[sp.next as usize];
        let pt = sp.next_loc;

        let ch = match ch {
            '\r' | '\n' => {
                sp.next += 1;

                if sp.next < file.text.len() {
                    let next_ch = file.text[sp.next as usize];
                    if (ch == '\r' && next_ch == '\n') || (ch == '\n' && next_ch == '\r') {
                        sp.next += 1;
                    }
                }

                sp.next_loc.col = 1;
                sp.next_loc.line += 1;

                '\n'
            },
            ch => {
                sp.next += 1;
                sp.next_loc.col += 1;
                ch
            },
        };

        (sp, SourceChar{ ch, pt,switched: false })
    }

}

impl Iterator for Source {
    type Item = SourceChar;

    /// Get the next source character, handling nested files.
    /// 
    fn next(&mut self) -> Option<Self::Item> {
        //
        // It is an invariant that either the pointer is at a valid 
        // character, or the entire stream is done. 
        //
        if self.iters.is_empty() {
            None 
        } else {            
            let sp = self.iters.last().unwrap();
            let file = &self.files[sp.file as usize];
            let switched = self.switched;
            
            let (sp, ch) = Source::extract_one_char(file, sp);

            let ch = if switched {
                println!("switched");
                self.switched = false;
                SourceChar{switched: true, ..ch}
            } else {
                println!("not switched");
                ch
            };


            *self.iters.last_mut().unwrap() = sp;
        
            self.pop_nested();

            Some(ch)
        }

    }   
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_characters() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['a', 'b', 'c'];

        source.push_data(&PathBuf::new(), text);

        assert!(matches!(source.next(), Some(SourceChar { ch: 'a', pt: Point{ file: 0, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'b', pt: Point{ file: 0, line: 1, col: 2 }, switched: false})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'c', pt: Point{ file: 0, line: 1, col: 3 }, switched: false})));
        assert!(matches!(source.next(), None));
        
        Ok(())
    }

    #[test]
    fn gets_characters_with_newline() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['a', '\n', 'c'];

        source.push_data(&PathBuf::new(), text);

        assert!(matches!(source.next(), Some(SourceChar { ch: 'a', pt: Point{ file: 0, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.next(), Some(SourceChar { ch: '\n', pt: Point{ file: 0, line: 1, col: 2 }, switched: false})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'c', pt: Point{ file: 0, line: 2, col: 1 }, switched: false})));
        assert!(matches!(source.next(), None));
        
        Ok(())
    }

    #[test]
    fn cr_lf_is_one_newline() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['a', '\r', '\n', 'c'];

        source.push_data(&PathBuf::new(), text);

        assert!(matches!(source.next(), Some(SourceChar { ch: 'a', pt: Point{ file: 0, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.next(), Some(SourceChar { ch: '\n', pt: Point{ file: 0, line: 1, col: 2 }, switched: false})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'c', pt: Point{ file: 0, line: 2, col: 1 }, switched: false})));
        assert!(matches!(source.next(), None));
        
        Ok(())
    }

    #[test]
    fn lf_cr_is_one_newline() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['a', '\n', '\r', 'c'];

        source.push_data(&PathBuf::new(), text);

        assert!(matches!(source.next(), Some(SourceChar { ch: 'a', pt: Point{ file: 0, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.next(), Some(SourceChar { ch: '\n', pt: Point{ file: 0, line: 1, col: 2 }, switched: false})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'c', pt: Point{ file: 0, line: 2, col: 1 }, switched: false})));
        assert!(matches!(source.next(), None));
        
        Ok(())
    }

    #[test]
    fn cr_counts_as_newline() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['a', '\r', 'c'];

        source.push_data(&PathBuf::new(), text);

        assert!(matches!(source.next(), Some(SourceChar { ch: 'a', pt: Point{ file: 0, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.next(), Some(SourceChar { ch: '\n', pt: Point{ file: 0, line: 1, col: 2 }, switched: false})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'c', pt: Point{ file: 0, line: 2, col: 1 }, switched: false})));
        assert!(matches!(source.next(), None));
        
        Ok(())
    }

    #[test]
    fn files_nest() -> Result<(), CcError> {
        let mut source = Source::new();
        let text1 = vec!['a', '\n', 'b'];
        let text2 = vec!['c', 'd', 'e'];

        source.push_data(&PathBuf::from("abc"), text1);
        assert!(matches!(source.next(), Some(SourceChar { ch: 'a', pt: Point{ file: 0, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.next(), Some(SourceChar { ch: '\n', pt: Point{ file: 0, line: 1, col: 2 }, switched: false})));
        source.push_data(&PathBuf::from("def"), text2);
        assert!(matches!(source.next(), Some(SourceChar { ch: 'c', pt: Point{ file: 1, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'd', pt: Point{ file: 1, line: 1, col: 2 }, switched: false})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'e', pt: Point{ file: 1, line: 1, col: 3 }, switched: false})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'b', pt: Point{ file: 0, line: 2, col: 1 }, switched: true})));
        assert!(matches!(source.next(), None));

        Ok(())
    }

    #[test]
    fn peek_multiple() -> Result<(), CcError> {

        let mut source = Source::new();
        let text1 = vec!['a', '\n', 'b'];
        let text2 = vec!['c', 'd', 'e'];

        source.push_data(&PathBuf::from("abc"), text1);
        assert!(matches!(source.next(), Some(SourceChar { ch: 'a', pt: Point{ file: 0, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.next(), Some(SourceChar { ch: '\n', pt: Point{ file: 0, line: 1, col: 2 }, switched: false})));
        source.push_data(&PathBuf::from("def"), text2);

        assert!(matches!(source.peek(), Some(SourceChar { ch: 'c', pt: Point{ file: 1, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.peek_n(0), Some(SourceChar { ch: 'c', pt: Point{ file: 1, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.peek_n(1), Some(SourceChar { ch: 'd', pt: Point{ file: 1, line: 1, col: 2 }, switched: false})));
        assert!(matches!(source.peek_n(3), Some(SourceChar { ch: 'b', pt: Point{ file: 0, line: 2, col: 1 }, switched: true})));

        assert!(matches!(source.next(), Some(SourceChar { ch: 'c', pt: Point{ file: 1, line: 1, col: 1 }, switched: true})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'd', pt: Point{ file: 1, line: 1, col: 2 }, switched: false})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'e', pt: Point{ file: 1, line: 1, col: 3 }, switched: false})));
        assert!(matches!(source.next(), Some(SourceChar { ch: 'b', pt: Point{ file: 0, line: 2, col: 1 }, switched: true})));
        assert!(matches!(source.next(), None));

        Ok(())
    }
}


/***
 * 

impl<F> Iterator for TakeWhile<'_, F> 
    where F: Fn(char) -> bool
{
    type Item = SourceChar;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.peek() {
            Some(sch) => {
                if (self.pred)(sch.ch) {
                    self.source.next()
                } else {
                    //
                    // Character did not pass predicate
                    //
                    None
                }
            },
            
            //
            // End of inner iterator
            //
            None => None,
        }
    }
}

/// Like TakeWhile, but returns at most `maxlen` characters
/// 
pub struct TakeWhileN<'a, F> 
    where F: Fn(char) -> bool
{
    /// The condition for iterating.
    pred: F, 

    /// The maximum number of characters to return
    maxlen: u32,

    /// The character count returned so far
    so_far: u32,

    /// The source code.
    source: &'a mut SourceFile,
}

impl<F> Iterator for TakeWhileN<'_, F> 
    where F: Fn(char) -> bool
{
    type Item = SourceChar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.so_far >= self.maxlen {
            None
        } else {
            match self.source.peek() {
                Some(sch) => {
                    if (self.pred)(sch.ch) {
                        self.so_far += 1;
                        self.source.next()
                    } else {
                        //
                        // Character did not pass predicate
                        //
                        None
                    }
                },
                
                //
                // End of inner iterator
                //
                None => None,
            }
        }
    }
}

impl SourceFile {
    /// Construct from the source as an exploded vector of characters
    /// 
    pub fn new(source: Vec<char>, file: u32) -> Self {
        SourceFile {
            text: source.into(),
            file,
            next_loc: Point{ file, line: 1, col: 1 },
        }
    }

    /// Skip whitespace in the input stream.
    /// 
    pub fn skip_whitespace(&mut self) {
        loop {
            if let Some(sch) = self.peek() {
                if sch.ch.is_whitespace() {
                    self.next();
                    continue;
                }
            }

            break;
        }
    }
    

    /// Peek the next character, if there is one.
    /// 
    pub fn peek(&self) -> Option<SourceChar> {
        self.text.front().map(|ch| SourceChar{ ch: *ch, pt: self.next_loc })
    }

    /// Return an iterator which will return characters as long as pred(ch) is true.
    /// 
    pub fn take_while<F: Fn(char) -> bool> (&mut self, pred: F) ->TakeWhile<'_, F> {
        TakeWhile {
            pred,
            source: self
        }
    }

    /// Return an iterator which will return characters as long as pred(ch) is true,
    /// for at most n characters.
    /// 
    pub fn take_while_n<F: Fn(char) -> bool> (&mut self, pred: F, n: u32) ->TakeWhileN<'_, F> {
        TakeWhileN {
            pred,
            maxlen: n,
            so_far: 0,
            source: self
        }
    }
}

impl Iterator for SourceFile {
    type Item = SourceChar;

    /// Iterate, but while iterating, keep track of the location in the source.
    /// 
    fn next(&mut self) -> Option<Self::Item> {
        self.text.pop_front().map(|ch| {
            let pt = self.next_loc;
            
            if ch == '\n' {
                self.next_loc.line += 1;
                self.next_loc.col = 1;
            } else {
                self.next_loc.col += 1;
            }

            SourceChar{ ch, pt }
        })
    }
}

/// Source, including all included source files.
/// 
pub struct Source {
    /// All source files
    pub files: Vec<SourceFile>,    
    


}


    **/