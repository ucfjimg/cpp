//
// Wrapper for iterating over the source code, as a stream of
// characters with source location attached.
//
use std::collections::VecDeque;

/// A location in the source code, for errors.
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    /// The 1-based line number in the source.
    pub line: i32,

    /// The 1-based column in the source.
    pub col: i32,
}

/// The source code in iterable form.
/// 
pub struct Source {
    /// The part of the source what is not yet processed.
    text: VecDeque<char>,

    /// The coordinates of the next character in the original file.
    next_loc: Point,
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
    source: &'a mut Source,
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

impl Source {
    /// Construct from the source as an exploded vector of characters
    /// 
    pub fn new(source: Vec<char>) -> Self {
        Source {
            text: source.into(),
            next_loc: Point{ line: 1, col: 1},
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

impl Iterator for Source {
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

