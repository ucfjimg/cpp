//
// Wrapper for iterating over the source code, as a stream of
// characters with source location attached.
//
use crate::ccerror::CcError;
use std::path::PathBuf;

/// A location in the source code, for errors.
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
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
}

/// The state for reading characters across all source files.
/// 
pub struct Source {
    /// All files, indexed by a file integer.
    pub files: Vec<SourceFile>,

    /// Nested stack of file pointers. The first will be the main
    /// source file.
    pub iters: Vec<SourcePointer>,    
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

        self.pop_nested();

        Ok(())
    } 

    fn push_data(&mut self, name: &PathBuf, text: Vec<char>) {
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

        self.pop_nested();
    }

    fn pop_nested(&mut self) {
        loop {
            match self.iters.last() {
                Some(sp) => {
                    if sp.next < self.files[sp.file as usize].text.len() {
                        break;
                    }
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

            Some(SourceChar{ ch, pt })
        }
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
            let sp = self.iters.last_mut().unwrap();
            let file = &self.files[sp.file as usize];
            assert!(sp.next < file.text.len());
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
                        if (ch == '\r' || next_ch == '\n') || (ch == '\n' || next_ch == '\r') {
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

            self.pop_nested();

            Some(SourceChar{ ch, pt })
        }

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