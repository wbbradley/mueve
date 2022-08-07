#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Lexeme {
    Integer(i64),
    Float(f64),
    Identifier(String),
    QuotedString(String),
    Operator(String),
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    Colon,
    EndOfLine
}

pub struct Location {
    filename: &str,
    line: i32,
    col: i32,
}

pub struct Token {
    location: Location,
    lexeme: Lexeme,
}

pub struct Lexer<'a> {
    text: mut &'a str,
}

impl<'a> Lexer<'a> {
    fn next_token<'a>(self) -> Lexeme
}
