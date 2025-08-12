use crate::ccerror::CcError;
use crate::source::Source;

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

    // other
    
    Eof
}

pub fn next_token(source: &mut Source) -> Result<PpToken, CcError> {
    

}

 