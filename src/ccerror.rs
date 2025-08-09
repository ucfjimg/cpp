use std::error::Error;
use std::fmt::Display;

use crate::source::Point;

/// Any preprocessor error.
/// 
#[derive(Debug, PartialEq)]
pub struct CcError {
    pub what: String,
    pub loc: Option<Point>,
}

impl CcError {
    /// Construct from a string.
    /// 
    pub fn new(what: String) -> Self {
        CcError {
            what,
            loc: None,
        }
    }

    /// Construct from a literal.
    ///
    pub fn from_str(what: &'static str) -> Self {
        CcError {
            what: what.to_owned(),
            loc: None,
        }
    }

    /// Construct from a string with an associated source code location.
    /// 
    pub fn err_with_loc(what: String, loc: Point) -> Self {
        CcError {
            what,
            loc: Some(loc),
        }
    }
}

impl Display for CcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pt) = self.loc {
            write!(f, "{}:{}: ", pt.line, pt.col)?;
        }
        write!(f, "{}", self.what)
    }
}

impl Error for CcError {
}

impl From<std::io::Error> for CcError {
    fn from(e: std::io::Error) -> Self {
        CcError::new(e.to_string())
    }
}
