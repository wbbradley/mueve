use crate::lexer::Lexeme;
use crate::location::Location;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Token<'a> {
    pub location: Location<'a>,
    pub lexeme: Lexeme<'a>,
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.lexeme)
    }
}
