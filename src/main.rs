use std::fmt;
mod lexer;
use lexer::{Lexeme, Lexer, Location, Token};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Identifier<'a> {
    name: &'a str,
    location: Location<'a>,
}

#[allow(dead_code)]
enum Predicate {
    Irrefutable(Id),
    Integer(i64),
    String(String),
    Tuple { dims: Vec<Box<Predicate>> },
}

#[allow(dead_code)]
pub struct PatternExpr {
    predicate: Predicate,
    expr: Expr,
}

#[allow(dead_code)]
pub struct Id {
    name: String,
}

#[allow(dead_code)]
pub enum Expr {
    Lambda {
        param_names: Vec<Id>,
        body: Box<Expr>,
    },
    LiteralInteger {
        value: i64,
    },
    LiteralString {
        value: String,
    },
    Id {
        name: String,
    },
    Match {
        subject: Box<Expr>,
        pattern_exprs: Vec<PatternExpr>,
    },
    Callsite {
        function: Box<Expr>,
        arguments: Vec<Box<Expr>>,
    },
    TupleCtor {
        dims: Vec<Box<Expr>>,
    },
}

#[allow(dead_code)]
fn mkcallsite(_exprs: Vec<Expr>) -> Expr {
    Expr::Lambda {
        param_names: Vec::new(),
        body: Box::new(Expr::Id {
            name: "foo".to_string(),
        }),
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Decl<'a> {
    id: Identifier<'a>,
    // pattern: Vec<Predicate>,
    // body: Expr,
}

#[derive(Debug)]
pub enum ErrorLevel {
    Info,
    Warning,
    Error,
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

#[derive(Debug)]
pub struct ParseError<'a> {
    location: Location<'a>,
    level: ErrorLevel,
    message: String,
}

impl<'a> std::error::Error for ParseError<'a> {}

impl<'a> ParseError<'a> {
    pub fn unexpected(token: Token<'a>, expected: &str) -> ParseError<'a> {
        ParseError {
            location: token.location.clone(),
            level: ErrorLevel::Error,
            message: format!("unexpected token ({token}) found. expected {expected}"),
        }
    }
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}: {}", self.location, self.level, self.message)
    }
}

fn maybe_id(lexer: Lexer) -> (Option<Identifier>, Lexer) {
    match lexer.peek() {
        (None, lexer) => (None, lexer.advance()),
        (
            Some(Token {
                location,
                lexeme: Lexeme::Identifier(name),
            }),
            lexer,
        ) => (
            Some(Identifier {
                name: name,
                location: location.clone(),
            }),
            lexer.advance(),
        ),
        (Some(_), lexer) => (None, lexer),
    }
}

fn parse_decl(lexer: Lexer) -> Result<(Option<Decl>, Lexer), ParseError> {
    match maybe_id(lexer) {
        (Some(id), lexer) => Ok((Some(Decl { id: id }), lexer)),
        (None, lexer) => Ok((None, lexer)),
    }
}

fn parse_decls(mut lexer: Lexer) -> Result<(Vec<Decl>, Lexer), ParseError> {
    let mut decls = Vec::new();
    loop {
        match parse_decl(lexer) {
            Ok((Some(decl), new_lexer)) => {
                lexer = new_lexer;
                decls.push(decl);
            }
            Ok((None, lexer)) => return Ok((decls, lexer)),
            Err(err) => return Err(err),
        }
    }
}

fn main() {
    let input = "fan 123454 14 \n pi.(} &";
    let lexer = Lexer::new("raw-text", &input);
    let (decls, _) = parse_decls(lexer.advance()).unwrap();
    println!("{:?}", decls);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lex_some() {}
}
