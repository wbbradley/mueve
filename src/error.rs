use crate::location::Location;
use crate::token::Token;
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ErrorLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct ParseError<'a> {
    location: Location<'a>,
    level: ErrorLevel,
    message: String,
}

impl std::fmt::Display for ErrorLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ErrorLevel::Info => "info",
                ErrorLevel::Warning => "warning",
                ErrorLevel::Error => "error",
            }
        )
    }
}

impl<'a> std::error::Error for ParseError<'a> {}

impl<'a> ParseError<'a> {
    pub fn error<T>(location: Location<'a>, message: T) -> ParseError<'a>
    where
        T: fmt::Display,
    {
        ParseError {
            location: location,
            level: ErrorLevel::Error,
            message: format!("{}", message),
        }
    }

    pub fn not_impl(location: Location<'a>) -> ParseError<'a> {
        ParseError {
            location: location,
            level: ErrorLevel::Error,
            message: "parsing this is not implemented".to_string(),
        }
    }

    pub fn unexpected<T>(token: Token<'a>, expected: T) -> ParseError<'a>
    where
        T: fmt::Display,
    {
        ParseError {
            location: token.location.clone(),
            level: ErrorLevel::Error,
            message: format!("unexpected token ({token}) found. expected {}", expected),
        }
    }
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}: {}", self.location, self.level, self.message)
    }
}
