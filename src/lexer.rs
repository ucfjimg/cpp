use crate::ccerror::CcError;
use crate::source::{Source, SourceChar, Point};

use std::collections::HashMap;

use lazy_static::lazy_static;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum PpToken {
    Identifier(String),
    StringLiteral(String),
    Number(String),
    CharLiteral(String),

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

/// Return the next lexical token in the input stream. 
/// 
/// Any whitespace before the token will be appended to the `emit` vector.
/// 
pub fn next_token(source: &mut Source, emit: &mut Vec<char>) -> Result<PpToken, CcError> {
    let mut newline = false;
    
    //
    // Whitespace
    //
    loop {
        let ch = match peek_spliced(source) {
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
        // Identifier?
        //
        if ch.ch.is_ascii_alphabetic() || ch.ch == '_' {
            return Ok(identifier(source));
        }
        
        //
        // Number? We want to look for this before an operator 
        // since .<digit> is the start of a pp-number, but otherwise
        // . is an operator.
        //
        let inum = if ch.ch == '.' { 1 } else { 0 };
        let is_number = match peek_spliced_n(source, inum) {
            Some(ch) => {
                ch.ch.is_ascii_digit()
            },
            _ => false
        };

        if is_number {
            return Ok(ppnumber(source));
        }

        //
        // Character literal?
        //
        if ch.ch == '\'' {
            next_spliced(source);
            return textlit(source, true, ch.pt);
        }

        //
        // String literal?
        //
        if ch.ch == '\"' {
            next_spliced(source);
            return textlit(source, false, ch.pt);
        }

        //
        // Operator?    
        //    
        match lookup_op(source, &OPERATORS) {
            Some(PpToken::BlockComment) => {
                skip_block_comment(source, ch.pt)?;
                emit.push(' ');
                continue;
            },
            Some(PpToken::LineComment) => {
                skip_line_comment(source, ch.pt)?;
                emit.push(' ');
                continue;
            },
            Some(op) => return Ok(op),
            None => {}, 
        };

        match peek_spliced(source) {
            Some(ch) => {
                next_spliced(source);
                return Ok(PpToken::Other(ch.ch));
            },
            _ => break,
        }
    }

    Ok(PpToken::Eof)
}

/// Collect an identifier. The caller must have verified that the next 
/// character in the source is a valid identifier start.
/// 
fn identifier(source: &mut Source) -> PpToken {
    let mut idchars = Vec::new();

    loop {
        let ch = match peek_spliced(source) {
            Some(ch) => ch.ch,
            None => break,
        };

        if ch.is_ascii_alphanumeric() || ch == '_' {
            idchars.push(ch);
        } else {
            break;
        }

        next_spliced(source);
    }

    let id: String = idchars.into_iter().collect();

    PpToken::Identifier(id)
}

/// Collect an number. The caller must have verified that the next 
/// character in the source is a valid number start.
/// 
/// Note that the production rules for a pp-number generate all valid
/// integer and float constants, including size and signed-ness suffixes,
/// but also match many sequences which are not valid numeric constants
/// by the more strict rules of later phases.
/// 
fn ppnumber(source: &mut Source) -> PpToken {
    //
    // The caller has checked that we have . or .<digit>
    // So we can just collect the rest of valid pp-number characters
    //
    let mut numchars = Vec::new();

    loop {
        let ch = match peek_spliced(source) {
            Some(ch) => ch.ch,
            None => break,
        };

        //
        // 'e' or 'E' can be followed by a number
        //
        if ch == 'e' || ch == 'E' {
            numchars.push(ch);
            next_spliced(source);

            match peek_spliced(source) {
                Some(ch) if ch.ch == '+' || ch.ch == '-' => {
                    numchars.push(ch.ch);
                    next_spliced(source);
                },
                _ => {},
            };

            continue;
        }

        //
        // Otherwise, check for valid character to append
        //
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
            numchars.push(ch);
            next_spliced(source);
        } else {
            break;
        }
    }

    PpToken::Number(numchars.into_iter().collect())
}

/// Collect a character or a string literal.
/// 
fn textlit(source: &mut Source, is_char: bool, pt: Point) -> Result<PpToken, CcError> {
    let mut chars = Vec::new();

    loop {
        match peek_spliced(source) {
            Some(ch) => {
                match ch.ch {
                    '\n' => {
                        return Err(
                            CcError::err_with_loc(
                                "unterminated character constant".to_string(), 
                                pt
                            )
                        )
                    }, 
                    '\'' if is_char => {
                        next_spliced(source);
                        break;
                    },
                    '"' if !is_char => {
                        next_spliced(source);
                        break;
                    },
                    '\\' => {
                        next_spliced(source);
                        escape_sequence(source, &mut chars, pt)?;
                    },
                    ch => {
                        chars.push(ch);
                        next_spliced(source);
                    }
                };
            },
            _ => {
                return Err(
                    CcError::err_with_loc(
                        "unterminated character constant".to_string(), 
                        pt
                    )
                )
            },
        }
    }
    
    if is_char {
        Ok(PpToken::CharLiteral(chars.into_iter().collect()))
    } else {
        Ok(PpToken::StringLiteral(chars.into_iter().collect()))
    }
}

/// Collect an escape sequence inside a character or string literal.
/// 
fn escape_sequence(source: &mut Source, accum: &mut Vec<char>, pt: Point) -> Result<(), CcError> {
    
    //
    // Note that the preprocessor is not responsible for converting escape
    // sequences, it just needs to know enough to parse character and string
    // constants with embedded quotes.
    //    
    accum.push('\\');
    match peek_spliced(source) {
        Some(ch) => {
            match ch.ch {
                'x' => {
                    accum.push('x');
                    next_spliced(source);

                    loop {
                        let ch = match peek_spliced(source) {
                            Some(ch) => ch.ch,
                            None => {
                                return Err(
                                    CcError::err_with_loc(
                                        "unterminated escape sequence".to_string(),
                                        pt
                                    )
                                )
                            }
                        };

                        if !ch.is_ascii_hexdigit() {
                            break;
                        }

                        accum.push(ch);
                        next_spliced(source);
                    }
                },
                '0'..='7' => {
                    accum.push(ch.ch);

                    loop {
                        let ch = match peek_spliced(source) {
                            Some(ch) => ch.ch,
                            None => {
                                return Err(
                                    CcError::err_with_loc(
                                        "unterminated escape sequence".to_string(),
                                        pt
                                    )
                                )
                            }
                        };

                        if !ch.is_ascii_digit() && ch != '8' && ch != '9' {
                            break;
                        }

                        accum.push(ch);
                        next_spliced(source);
                    }
                },
                ch => {
                    accum.push(ch);
                    next_spliced(source);
                }
            }
        },
        _ => return Err(
            CcError::err_with_loc(
                "unterminated escape sequence".to_string(),
                pt
            )
        )
    }

    Ok(())
}

/// Given that the lead characters of a block comment (i.e. /*) have been
/// consumed, scan and discard source until a comment end sequence (*/) is
/// found. 
///
fn skip_block_comment(source: &mut Source, loc: Point) -> Result<(), CcError> {
    let mut last_star = false;
    
    loop {
        let ch = match next_spliced(source) {            
            Some(ch) => {
                match ch.ch {
                    '*' => last_star = true,
                    '/' => if last_star {
                        break;
                    },
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

/// Given that the lead characters of a line comment (i.e. //) have been
/// consumed, scan and discard source until the end of the line.
///
fn skip_line_comment(source: &mut Source, loc: Point) -> Result<(), CcError> {
    loop {
        match next_spliced(source) {
            Some(ch) if ch.ch == '\n' => {
                break;
            },
            None => {
                next_spliced(source);
                break;
            },
            _ => {},            
        }
    }

    Ok(())
}

/// Consume and return the next character in the source stream, handling line splicing.
/// 
fn next_spliced(source: &mut Source) -> Option<SourceChar> {
    loop {
        match source.peek() {
            Some(ch) if ch.ch == '\\' => {
                let backslash = ch;
                source.next();

                match source.peek() {
                    Some(ch) if ch.ch == '\n' => {
                        source.next();
                        continue;
                    },
                    _ => break Some(backslash)
                }
            },
            _ => break source.next()
        }
    }
}

/// Return the next character in the source stream without consuming it, 
/// handling line splicing.
/// 
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

/// Return the n'th next character in the source stream without consuming it, 
/// handling line splicing.
/// 
/// If n is zero, the immediate next character is returned.
/// 
fn peek_spliced_n(source: &Source, mut n: u32) -> Option<SourceChar> {
    let mut i : u32 = 0;

    loop {
        match source.peek_n(i) {
            Some(ch) if ch.ch == '\\' => {
                let backslash = ch;

                match source.peek_n(i+1) {
                    Some(ch) if ch.ch == '\n' => {
                        i += 2;
                        continue;
                    },
                    _ => break Some(backslash)
                }
            },
            _ => {
                if n == 0 {
                    break source.peek_n(i);
                } else {
                    n = n - 1;
                    i = i + 1;
                }
            }
        }
    }
}
 
/// Walk, recursively, the OPERATORS table to translate the longest substring
/// of `source` that is a valid operator.
///
fn lookup_op(source: &mut Source, map: &HashMap<char, OpNode>) -> Option<PpToken> {
    match peek_spliced(source) {
        Some(sch) => {
            match map.get(&sch.ch) {
                Some(op) => {
                    next_spliced(source);

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

        let mut source = Source::new();
        let text = vec!['/', '/', ' ', '*', '\\', '\n', '=', '\n', '*'];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        let token = next_token(&mut source, &mut emit)?;

        assert_eq!(emit, vec![' ']);
        assert_eq!(token, PpToken::Star);

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

    #[test]
    fn peeks_past_multiple_splices() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['\\', '\n', '+', '\\', '\n', '*'];

        source.push_data(&PathBuf::from("abc"), text);

        assert!(matches!(peek_spliced(&source), Some(SourceChar{ch: '+', pt: Point { file: 0, line: 2, col: 1 } })));
        assert!(matches!(peek_spliced_n(&source, 1), Some(SourceChar{ch: '*', pt: Point { file: 0, line: 3, col: 1 } })));

        Ok(())
    }
    #[test]
    fn identifier() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['a', 'b', 'c', '+', 'x'];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();

        let id = PpToken::Identifier("abc".to_string());
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Add));
        let id = PpToken::Identifier("x".to_string());
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));

        Ok(())
    }

    #[test]
    fn dot_is_an_operator() -> Result<(), CcError> {
        let mut source = Source::new();

        //
        // '.', not followed by a digit, is an operator.
        //         
        let text = vec!['.', 'b'];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();

        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Dot));
        let id = PpToken::Identifier("b".to_string());
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));
        Ok(())
    }

    #[test]
    fn numbers() -> Result<(), CcError> {
        //
        // . followed by a digit starts a pp-number
        //
        let mut source = Source::new();
        let text = vec!['.', '3', '1', 'e', '-', '0', ','];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        
        let id = PpToken::Number(".31e-0".to_string());
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Comma));
        
        //
        // A digit starts a pp-number
        //
        let mut source = Source::new();
        let text = vec!['3', '1', '4', '1', '6', ','];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        
        let id = PpToken::Number("31416".to_string());
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Comma));
        Ok(())
    }

    #[test]
    fn char_const() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['\'', 'a', '\'', ','];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        
        let id = PpToken::CharLiteral("a".to_string());
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Comma));

        Ok(())
    }

    #[test]
    fn unterminated_char_const() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['\'', 'a', '\n', ','];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        
        assert!(next_token(&mut source, &mut emit).is_err());
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Comma));

        Ok(())
    }

    #[test]
    fn char_const_escaped_quote() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['\'', '\\', '\'', '\'', ','];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        
        let id = PpToken::CharLiteral("\\'".to_string());
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Comma));

        Ok(())
    }

    #[test]
    fn str_const() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['\"', 'a', 'b', 'c', '\"', ','];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        
        let id = PpToken::StringLiteral("abc".to_string());
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Comma));

        Ok(())
    }

    #[test]
    fn unterminated_str_const() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['\"', 'a', '\n', ','];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        
        assert!(next_token(&mut source, &mut emit).is_err());
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Comma));

        Ok(())
    }

    #[test]
    fn str_const_escaped_quote() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['"', '\\', '"', '"', ','];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        
        let id = PpToken::StringLiteral("\\\"".to_string());
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Comma));

        Ok(())
    }

    #[test]
    fn random_character_are_other() -> Result<(), CcError> {
        let mut source = Source::new();
        let text = vec!['$', ','];

        source.push_data(&PathBuf::from("abc"), text);

        let mut emit = Vec::new();
        
        let id = PpToken::Other('$');
        assert_eq!(next_token(&mut source, &mut emit), Ok(id));
        assert_eq!(next_token(&mut source, &mut emit), Ok(PpToken::Comma));

        Ok(())
    }
}
