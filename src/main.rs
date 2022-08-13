use std::fmt;

use crate::error::ParseError;
use crate::lexer::{Lexeme, Lexer};
use crate::location::Location;
use crate::token::Token;

mod error;
mod lexer;
mod location;
mod token;

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

fn parse_predicate(lexer: Lexer) -> Result<(Option<Predicate>, Lexer), ParseError> {
    Err(ParseError::error(lexer.location, "foo"))
}

fn parse_predicates(mut lexer: Lexer) -> Result<(Vec<Predicate>, Lexer), ParseError> {
    let mut predicates = Vec::new();
    loop {
        match parse_predicate(lexer) {
            Ok((None, lexer)) => return Ok((predicates, lexer)),
            Ok((Some(predicate), next_lexer)) => {
                predicates.push(predicate);
                lexer = next_lexer.advance();
            }
            Err(err) => return Err(err),
        }
    }
}

fn parse_decl(lexer: Lexer) -> Result<(Option<Decl>, Lexer), ParseError> {
    let (id, lexer) = match maybe_id(lexer) {
        (Some(id), lexer) => (id, lexer),
        (None, lexer) => return Ok((None, lexer)),
    };
    let (_predicates, next_lexer) = parse_predicates(lexer.advance())?;
    let next_lexer = next_lexer.chomp(Lexeme::Assign)?;
    Ok((Some(Decl { id: id }), next_lexer))
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
    match parse_decls(lexer.advance()) {
        Ok((decls, _)) => println!("Ok {:?}", decls),
        Err(err) => println!("{}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lex_some() {}
}
