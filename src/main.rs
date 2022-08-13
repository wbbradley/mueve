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
#[derive(Debug, Clone)]
enum Predicate<'a> {
    Irrefutable(Identifier<'a>),
    Integer(i64),
    String(String),
    Tuple { dims: Vec<Box<Predicate<'a>>> },
}

#[allow(dead_code)]
pub struct PatternExpr<'a> {
    predicate: Predicate<'a>,
    expr: Expr<'a>,
}

#[allow(dead_code)]
pub enum Expr<'a> {
    Lambda {
        param_names: Vec<Identifier<'a>>,
        body: Box<Expr<'a>>,
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
        subject: Box<Expr<'a>>,
        pattern_exprs: Vec<PatternExpr<'a>>,
    },
    Callsite {
        function: Box<Expr<'a>>,
        arguments: Vec<Box<Expr<'a>>>,
    },
    TupleCtor {
        dims: Vec<Box<Expr<'a>>>,
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
    predicates: Vec<Predicate<'a>>,
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

fn parse_tuple_predicate<'a>(
    location: Location<'a>,
    _lexer: Lexer<'a>,
) -> Result<(Option<Predicate<'a>>, Lexer<'a>), ParseError<'a>> {
    return Err(ParseError::error(location, "failed to parse tuple"));
}

fn parse_predicate(lexer: Lexer) -> Result<(Option<Predicate>, Lexer), ParseError> {
    match lexer.peek() {
        (Some(token), lexer) => match token.lexeme {
            Lexeme::Signed(value) => Ok((Some(Predicate::Integer(value)), lexer.advance())),
            Lexeme::QuotedString(value) => {
                Ok((Some(Predicate::String(value.to_string())), lexer.advance()))
            }
            Lexeme::Identifier(name) => Ok((
                Some(Predicate::Irrefutable(Identifier {
                    name: name,
                    location: lexer.location.clone(),
                })),
                lexer.advance(),
            )),
            Lexeme::LParen => parse_tuple_predicate(token.location, lexer),
            _ => Ok((None, lexer)),
        },
        (None, lexer) => {
            return Err(ParseError::error(
                lexer.location,
                "missing token where a predicate was expected?",
            ))
        }
    }
}

fn parse_predicates(mut lexer: Lexer) -> Result<(Vec<Predicate>, Lexer), ParseError> {
    let mut predicates = Vec::new();
    loop {
        match parse_predicate(lexer)? {
            (None, lexer) => return Ok((predicates, lexer)),
            (Some(predicate), next_lexer) => {
                println!("found predicate {:?}", predicate);
                predicates.push(predicate);
                lexer = next_lexer;
            }
        }
    }
}

fn parse_decl(lexer: Lexer) -> Result<(Option<Decl>, Lexer), ParseError> {
    let (id, lexer) = match maybe_id(lexer) {
        (Some(id), lexer) => (id, lexer),
        (None, lexer) => return Ok((None, lexer)),
    };
    let (predicates, lexer) = parse_predicates(lexer)?;
    let next_lexer = lexer.chomp(Lexeme::Assign)?;

    Ok((
        Some({
            let decl = Decl {
                id: id,
                predicates: predicates,
            };
            println!("found decl {:?}", decl);
            decl
        }),
        next_lexer,
    ))
}

fn parse_decls(mut lexer: Lexer) -> Result<(Vec<Decl>, Lexer), ParseError> {
    let mut decls = Vec::new();
    loop {
        match parse_decl(lexer)? {
            (Some(decl), new_lexer) => {
                lexer = new_lexer;
                decls.push(decl);
            }
            (None, lexer) => return Ok((decls, lexer)),
        }
    }
}

fn main() {
    let input = "fan 123454 14 \n pi=(} &";
    println!("parsing '{}'...", input);
    let lexer = Lexer::new("raw-text", &input);
    match parse_decls(lexer.advance()) {
        Ok((decls, _)) => println!("Parsed {:?}", decls),
        Err(err) => println!("{}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lex_some() {}
}
