use crate::ccerror::CcError;
use crate::source::{Source, SourceChar, Point};

use std::collections::HashMap;

use lazy_static::lazy_static;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum PpToken {
    Identifier(String),
    StringLiteral(Vec<char>),
    Number(String),
    CharLiteral(char),

    // operators

    Hash,
    Add,
    Subtract,
    Star,
    Divide,
    Mod,
    Increment,
    Decrement,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    LogicalNot,
    LogicalAnd,
    LogicalOr,
    BitNot,
    Ampersand,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LeftShiftAssign,
    RightShiftAssign,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Dot,
    Arrow,
    Semicolon,
    Question,
    Colon,
    Comma,

    // Any character that's not part of another token.
    Other(char),
    

    // never returned

    BlockComment,
    LineComment,

    // other
    
    Eof
}

lazy_static! {
    #[derive(Debug)]
    static ref OPERATORS: HashMap<char, OpNode> = vec![
        ('(', OpNode::new(PpToken::LeftParen, None)),
        (')', OpNode::new(PpToken::RightParen, None)),
        ('{', OpNode::new(PpToken::LeftBrace, None)),
        ('}', OpNode::new(PpToken::RightBrace, None)),
        ('[', OpNode::new(PpToken::LeftBracket, None)),
        (']', OpNode::new(PpToken::RightBracket, None)),
        (';', OpNode::new(PpToken::Semicolon, None)),
        ('#', OpNode::new(PpToken::Hash, None)),
        ('?', OpNode::new(PpToken::Question, None)),
        (':', OpNode::new(PpToken::Colon, None)),
        (',', OpNode::new(PpToken::Comma, None)),
        ('~', OpNode::new(PpToken::BitNot, None)),
        ('.', OpNode::new(PpToken::Dot, None)),
        ('+', OpNode::new(PpToken::Add, 
            Some(vec![
                ('+', OpNode::new(PpToken::Increment, None)),
                ('=', OpNode::new(PpToken::AddAssign, None)),
            ].into_iter().collect())   
        )),
        ('-', OpNode::new(PpToken::Subtract, 
            Some(vec![
                ('+', OpNode::new(PpToken::Decrement, None)),
                ('=', OpNode::new(PpToken::SubtractAssign, None)),
                ('>', OpNode::new(PpToken::Arrow, None)),
            ].into_iter().collect())   
        )),
        ('*', OpNode::new(PpToken::Star, 
            Some(vec![
                ('=', OpNode::new(PpToken::MultiplyAssign, None)),
            ].into_iter().collect())   
        )),
        ('/', OpNode::new(PpToken::Divide, 
            Some(vec![
                ('=', OpNode::new(PpToken::DivideAssign, None)),
                ('*', OpNode::new(PpToken::BlockComment, None)),
                ('/', OpNode::new(PpToken::LineComment, None)),
            ].into_iter().collect())   
        )),
        ('%', OpNode::new(PpToken::Mod, 
            Some(vec![
                ('=', OpNode::new(PpToken::ModAssign, None)),
            ].into_iter().collect())   
        )),
        ('=', OpNode::new(PpToken::Assign, 
            Some(vec![
                ('=', OpNode::new(PpToken::Equal, None)),
            ].into_iter().collect())   
        )),
        ('!', OpNode::new(PpToken::LogicalNot, 
            Some(vec![
                ('=', OpNode::new(PpToken::NotEqual, None)),
            ].into_iter().collect())   
        )),
        ('&', OpNode::new(PpToken::Ampersand, 
            Some(vec![
                ('=', OpNode::new(PpToken::AndAssign, None)),
                ('&', OpNode::new(PpToken::LogicalAnd, None)),
            ].into_iter().collect())   
        )),
        ('|', OpNode::new(PpToken::BitOr, 
            Some(vec![
                ('=', OpNode::new(PpToken::OrAssign, None)),
                ('|', OpNode::new(PpToken::LogicalOr, None)),
            ].into_iter().collect())   
        )),
        ('^', OpNode::new(PpToken::BitXor, 
            Some(vec![
                ('=', OpNode::new(PpToken::XorAssign, None)),
            ].into_iter().collect())   
        )),
        ('<', OpNode::new(PpToken::Less, 
            Some(vec![
                ('=', OpNode::new(PpToken::LessEqual, None)),
                ('<', OpNode::new(PpToken::ShiftLeft, 
                    Some(vec![
                        ('=', OpNode::new(PpToken::LeftShiftAssign, None)),                        
                    ].into_iter().collect())
                )),
            ].into_iter().collect())   
        )),
        ('>', OpNode::new(PpToken::Greater, 
            Some(vec![
                ('=', OpNode::new(PpToken::GreaterEqual, None)),
                ('>', OpNode::new(PpToken::ShiftRight, 
                    Some(vec![
                        ('=', OpNode::new(PpToken::RightShiftAssign, None)),                        
                    ].into_iter().collect())
                )),
            ].into_iter().collect())   
        )),

    ].into_iter().collect();
}

pub fn next_token(source: &mut Source, emit: &mut Vec<char>) -> Result<PpToken, CcError> {
    let mut newline = false;
    
    //
    // Whitespace
    //
    loop {
        let ch = match source.peek() {
            Some(ch) => ch,
            None => return Ok(PpToken::Eof)
        };

        if ch.ch.is_ascii_whitespace() {
            if ch.ch == '\n' {
                newline = true;
            }
            emit.push(ch.ch);
            source.next();
            continue;
        }
        
        //
        // Operator?    
        //    
        match lookup_op(source, &OPERATORS) {
            Some(PpToken::BlockComment) => {
                skip_block_comment(source, ch.pt)?;
                emit.push(' ');
            },
            Some(PpToken::LineComment) => {
                skip_line_comment(source, ch.pt)?;
                emit.push(' ');
            },
            Some(op) => return Ok(op),
            _ => break,
        };
    }

    Ok(PpToken::Eof)
}

fn skip_block_comment(source: &mut Source, loc: Point) -> Result<(), CcError> {
    let mut last_star = false;
    
    loop {
        let ch = match source.next() {
            Some(ch) => {
                match ch.ch {
                    '*' => last_star = true,
                    '/' => if last_star {
                        break;
                    },
                    '\\' => if matches!(source.peek(), Some(SourceChar{ch: '\n', pt: _})) {
                        source.next();
                        continue;
                    }
                    _ => last_star = false,
                }
            },
            None => return
                Err(
                    CcError::err_with_loc(
                        "unterminated block comment.".to_string(), 
                        loc
                    )
                ),
        };
    }
    Ok(())
}

fn skip_line_comment(source: &mut Source, loc: Point) -> Result<(), CcError> {
    loop {
        match source.next() {
            Some(ch) if ch.ch == '\n' => {
                break;
            },
            None => {
                source.next();
                break;
            },
            _ => {},            
        }
    }

    Ok(())
}

fn next_spliced(source: &mut Source) -> Option<SourceChar> {
    loop {
        match source.peek() {
            Some(ch) if ch.ch == '\\' => {
                let backslash = ch;
                source.next();

                match source.peek() {
                    Some(ch) if ch.ch == '\n' => continue,
                    _ => break Some(backslash)
                }
            },
            _ => break source.next()
        }
    }
}

fn peek_spliced(source: &Source) -> Option<SourceChar> {
    let mut n : u32 = 0;

    loop {
        match source.peek_n(n) {
            Some(ch) if ch.ch == '\\' => {
                let backslash = ch;

                match source.peek_n(n+1) {
                    Some(ch) if ch.ch == '\n' => {
                        n += 2;
                        continue;
                    },
                    _ => break Some(backslash)
                }
            },
            _ => break source.peek_n(n)
        }
    }
}

 
#[derive(Debug)]
struct OpNode {
    token: PpToken,
    next: Option<HashMap<char, OpNode>>,
}

impl OpNode {
    fn new(token: PpToken, next: Option<HashMap<char, OpNode>>) -> Self {
        OpNode {
            token,
            next
        }
    }    
}


/// Walk, recursively, the OPERATORS table to translate the longest next substring
/// of `source` that is a valid operator.
///
fn lookup_op(source: &mut Source, map: &HashMap<char, OpNode>) -> Option<PpToken> {
    match source.peek() {
        Some(sch) => {
            if sch.ch == '\\' {
                source.next();
                match source.peek() {
                    Some(ch) if ch.ch == '\n' => {
                        source.next();
                        return lookup_op(source, map)
                    },
                    _ => {}
                }

                //
                // Otherwise, stray backslash 
                //
                return Some(PpToken::Other('\\'));
            }


            match map.get(&sch.ch) {
                Some(op) => {
                    source.next();

                    if let Some(next) = &op.next { 
                       if let Some(token) = lookup_op(source, next) {
                            return Some(token)
                        }
                    }
                    
                    Some(op.token.clone())
                },
                None => None,
            }
        }
        None => None,
    }
}

#[cfg(test)] 
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn parses_operator() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec![' ', '=', '='];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        let token = next_token(&mut source, &mut emit)?;

        assert_eq!(emit, vec![' ']);
        assert_eq!(token, PpToken::Equal);

        Ok(())
    }

    #[test]
    fn parses_spliced() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec![' ', '=', '\\', '\n', '='];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        let token = next_token(&mut source, &mut emit)?;

        assert_eq!(emit, vec![' ']);
        assert_eq!(token, PpToken::Equal);

        Ok(())
    }

    #[test]
    fn skips_block_comment() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec![' ', '/', '*', '\n', '*', '/', '=', '='];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        let token = next_token(&mut source, &mut emit)?;

        assert_eq!(emit, vec![' ', ' ']);
        assert_eq!(token, PpToken::Equal);

        let text = vec![' ', '/', '*', '/', '\n', '*', '/', '=', '='];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        let token = next_token(&mut source, &mut emit)?;

        assert_eq!(emit, vec![' ', ' ']);
        assert_eq!(token, PpToken::Equal);
        Ok(())
    }

    #[test]
    fn skips_line_spliced_block_comment() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec![' ', '/', '*', '\n', '*', '\\', '\n', '/', '=', '='];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        let token = next_token(&mut source, &mut emit)?;

        assert_eq!(emit, vec![' ', ' ']);
        assert_eq!(token, PpToken::Equal);

        Ok(())
    }

    #[test]
    fn skips_line_comment() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec![' ', '/', '/',' ', ' ', '\n', '=', '='];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        let token = next_token(&mut source, &mut emit)?;

        assert_eq!(emit, vec![' ', ' ']);
        assert_eq!(token, PpToken::Equal);

        Ok(())
    }

    #[test]
    fn skips_line_spliced_line_comment() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec![' ', '/', '\\', '\n', '/', ' ', '\n', '=', '='];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        let token = next_token(&mut source, &mut emit)?;

        assert_eq!(emit, vec![' ', ' ']);
        assert_eq!(token, PpToken::Equal);

        Ok(())
    }

    #[test]
    fn peeks_past_splices() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['\\', '\n', '\\', '\n', '*'];

        source.push_data(&PathBuf::from("abc"), text);

        assert!(matches!(peek_spliced(&source), Some(SourceChar{ch: '*', pt: Point { file: 0, line: 3, col: 1 } })));

        Ok(())
    }
}
