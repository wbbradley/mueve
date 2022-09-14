use std::borrow::Borrow;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, PartialEq)]
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

pub trait HasLocation<'a> {
    fn get_location(&self) -> &Location<'a>;
}

impl<'a, T: HasLocation<'a>> HasLocation<'a> for Box<T> {
    fn get_location(&self) -> &Location<'a> {
        let borrowed: &T = self.borrow();
        borrowed.get_location()
    }
}
