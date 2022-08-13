use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Location<'a> {
    pub filename: &'a str,
    pub line: i32,
    pub col: i32,
}

impl<'a> std::fmt::Display for Location<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.col)
    }
}