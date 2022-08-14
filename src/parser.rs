use std::fmt;

use crate::error::ParseError;
use crate::identifier::Identifier;
use crate::lexer::{Lexeme, Lexer};
use crate::location::{HasLocation, Location};
use crate::token::Token;

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Predicate<'a> {
    Irrefutable(Identifier<'a>),
    Integer(i64),
    String(String),
    Tuple { dims: Vec<Box<Predicate<'a>>> },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PatternExpr<'a> {
    predicate: Predicate<'a>,
    expr: Expr<'a>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expr<'a> {
    Lambda {
        location: Location<'a>,
        param_names: Vec<Identifier<'a>>,
        body: Box<Expr<'a>>,
    },
    Let {
        location: Location<'a>,
        binding: Identifier<'a>,
        value: Box<Expr<'a>>,
        body: Box<Expr<'a>>,
    },
    LiteralInteger {
        location: Location<'a>,
        value: i64,
    },
    LiteralString {
        location: Location<'a>,
        value: String,
    },
    Symbol {
        id: Identifier<'a>,
    },
    Match {
        location: Location<'a>,
        subject: Box<Expr<'a>>,
        pattern_exprs: Vec<PatternExpr<'a>>,
    },
    Callsite {
        function: Box<Expr<'a>>,
        arguments: Vec<Box<Expr<'a>>>,
    },
    TupleCtor {
        location: Location<'a>,
        dims: Vec<Box<Expr<'a>>>,
    },
}

impl<'a> HasLocation<'a> for Expr<'a> {
    fn get_location(&self) -> &Location<'a> {
        match self {
            Expr::Lambda {
                location,
                param_names: _,
                body: _,
            } => &location,
            Expr::Let {
                location,
                binding: _,
                value: _,
                body: _,
            } => location,
            Expr::LiteralInteger { location, value: _ } => location,
            Expr::LiteralString { location, value: _ } => location,
            Expr::Symbol { id } => &id.location,
            Expr::Match {
                location,
                subject: _,
                pattern_exprs: _,
            } => location,
            Expr::Callsite {
                function,
                arguments: _,
            } => function.get_location(),
            Expr::TupleCtor { location, dims: _ } => location,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Decl<'a> {
    id: Identifier<'a>,
    predicates: Vec<Predicate<'a>>,
    body: Expr<'a>,
}

impl<'a> HasLocation<'a> for Decl<'a> {
    fn get_location(&self) -> &Location<'a> {
        &self.id.location
    }
}

#[derive(Debug)]
#[allow(dead_code)]
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
            Some(Identifier::new(name, location.clone())),
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
                Some(Predicate::Irrefutable(Identifier::new(
                    name,
                    lexer.location.clone(),
                ))),
                lexer.advance(),
            )),
            Lexeme::Operator("(") => parse_tuple_predicate(token.location, lexer),
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

fn parse_identifier(lexer: Lexer) -> Result<(Identifier, Lexer), ParseError> {
    match lexer.peek() {
        (
            Some(Token {
                location,
                lexeme: Lexeme::Identifier(name),
            }),
            lexer,
        ) => Ok((Identifier::new(name, location), lexer.advance())),
        (_, lexer) => Err(ParseError::error(
            lexer.location,
            "expected an identifier here",
        )),
    }
}

fn parse_let_expr<'a>(
    location: Location<'a>,
    lexer: Lexer<'a>,
) -> Result<(Option<Box<Expr<'a>>>, Lexer<'a>), ParseError<'a>> {
    let (binding_id, lexer) = parse_identifier(lexer)?;
    let lexer = lexer.chomp(Lexeme::Operator("="))?;
    let (binding_value, lexer) = parse_callsite(lexer)?;
    let lexer = lexer.chomp(Lexeme::Operator(":"))?;
    let (in_body, lexer) = parse_callsite(lexer)?;
    Ok((
        Some(
            Expr::Let {
                location: location,
                binding: binding_id,
                value: binding_value.into(),
                body: in_body.into(),
            }
            .into(),
        ),
        lexer,
    ))
}

fn parse_callsite_term(lexer: Lexer) -> Result<(Option<Box<Expr>>, Lexer), ParseError> {
    match lexer.peek() {
        (None, lexer) => Ok((None, lexer.advance())),
        (
            // A symbol reference.
            Some(Token {
                location,
                lexeme: Lexeme::Identifier(name),
            }),
            lexer,
        ) => {
            if name == "let" {
                parse_let_expr(lexer.location.clone(), lexer.advance())
            } else {
                Ok((
                    Some(
                        Expr::Symbol {
                            id: Identifier::new(name, location),
                        }
                        .into(),
                    ),
                    lexer.advance(),
                ))
            }
        }
        (
            Some(Token {
                location: _,
                lexeme: Lexeme::Semicolon,
            }),
            lexer,
        ) => Ok((None, lexer.advance())),
        (
            Some(Token {
                location: _,
                lexeme: Lexeme::Operator("="),
            }),
            lexer,
        ) => Ok((None, lexer)),
        (
            // Parentheses.
            Some(Token {
                location: _,
                lexeme: Lexeme::LParen,
            }),
            lexer,
        ) => {
            let (expr, lexer) = parse_callsite(lexer.advance())?;
            Ok((Some(expr.into()), lexer.chomp(Lexeme::RParen)?))
        }
        (
            // Parentheses.
            Some(Token {
                location: _,
                lexeme: Lexeme::RParen,
            }),
            lexer,
        ) => Ok((None, lexer)),
        (
            // An operator reference (which amounts to a symbol reference).
            Some(Token {
                location,
                lexeme: Lexeme::Operator(name),
            }),
            lexer,
        ) => Ok((
            Some(
                Expr::Symbol {
                    id: Identifier::new(name, location),
                }
                .into(),
            ),
            lexer.advance(),
        )),
        (x, lexer) => {
            let x = x.unwrap();
            eprintln!("{}: ran into {:?}", x.location, x);
            Err(ParseError::not_impl(lexer.location))
        }
    }
    /*
    parse_parentheses,
    parse_string_literal,
    parse_do_notation,
    parse_if_then,
    parse_match,
    parse_number,
    parse_identifier,
    parse_ctor,
    */
}

fn parse_callsite(lexer: Lexer) -> Result<(Expr, Lexer), ParseError> {
    let (maybe_function, lexer) = parse_callsite_term(lexer)?;
    match maybe_function {
        Some(function) => match parse_many(parse_callsite_term, lexer)? {
            (callsite_terms, lexer) => {
                if callsite_terms.len() == 0 {
                    return Err(ParseError::error(
                        lexer.location,
                        "missing an expression here?",
                    ));
                }
                Ok((
                    Expr::Callsite {
                        function: function,
                        arguments: callsite_terms,
                    },
                    lexer,
                ))
            }
        },
        None => Err(ParseError::error(
            lexer.location,
            "missing function callsite expression",
        )),
    }
}

pub fn parse_decl(lexer: Lexer) -> Result<(Option<Decl>, Lexer), ParseError> {
    let (id, lexer) = match maybe_id(lexer) {
        (Some(id), lexer) => (id, lexer),
        (None, lexer) => return Ok((None, lexer)),
    };
    let (predicates, lexer) = parse_predicates(lexer)?;
    let lexer = lexer.chomp(Lexeme::Operator("="))?;
    let (expr, lexer) = parse_callsite(lexer)?;
    println!("Found callsite {:?}", expr);
    Ok((
        Some({
            let decl = Decl {
                id: id,
                predicates: predicates,
                body: expr,
            };
            println!("{}: found decl {:?}", decl.get_location(), decl);
            decl
        }),
        lexer,
    ))
}

pub fn parse_many<'a, T, P>(
    parser: P,
    mut lexer: Lexer<'a>,
) -> Result<(Vec<T>, Lexer<'a>), ParseError<'a>>
where
    T: 'a + std::fmt::Debug + HasLocation<'a>,
    P: 'a + Fn(Lexer<'a>) -> Result<(Option<T>, Lexer<'a>), ParseError<'a>>,
{
    let mut objects = Vec::new();
    loop {
        match parser(lexer)? {
            (Some(object), new_lexer) => {
                lexer = new_lexer;
                // let loc = object.get_location();
                // println!("{}: info: found a thing! {:?}", loc, object);
                objects.push(object);
            }
            (None, lexer) => return Ok((objects, lexer)),
        }
    }
}
