use std::fmt;

use crate::error::{ParseError, ParseResult};
use crate::identifier::Identifier;
use crate::lexer::{Lexeme, Lexer};
use crate::location::{HasLocation, Location};
use crate::token::Token;

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Predicate<'a> {
    Irrefutable(Identifier<'a>),
    Integer {
        location: Location<'a>,
        value: i64,
    },
    String {
        location: Location<'a>,
        value: String,
    },
    Ctor {
        ctor_id: Identifier<'a>,
        dims: Vec<Box<Predicate<'a>>>,
    },
    Tuple {
        location: Location<'a>,
        dims: Vec<Box<Predicate<'a>>>,
    },
}

impl<'a> HasLocation<'a> for Predicate<'a> {
    fn get_location(&self) -> &Location<'a> {
        match self {
            Predicate::Irrefutable(id) => id.get_location(),
            Predicate::Integer { location, value: _ } => &location,
            Predicate::String { location, value: _ } => &location,
            Predicate::Ctor { ctor_id, dims: _ } => ctor_id.get_location(),
            Predicate::Tuple { location, dims: _ } => &location,
        }
    }
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
    LiteralFloat {
        location: Location<'a>,
        value: f64,
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
            Expr::LiteralFloat { location, value: _ } => location,
            Expr::LiteralString { location, value: _ } => location,
            Expr::Symbol { id } => id.get_location(),
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
    predicates: Vec<Box<Predicate<'a>>>,
    body: Expr<'a>,
}

impl<'a> HasLocation<'a> for Decl<'a> {
    fn get_location(&self) -> &Location<'a> {
        self.id.get_location()
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

fn is_keyword(name: &str) -> bool {
    name == "if"
        || name == "then"
        || name == "else"
        || name == "do"
        || name == "let"
        || name == "in"
}

fn maybe_id<'a>(lexer: &mut Lexer<'a>) -> ParseResult<'a, Option<Identifier<'a>>> {
    match lexer.peek() {
        None => {
            lexer.advance_mut()?;
            Ok(None)
        }
        Some(Token {
            location,
            lexeme: Lexeme::Identifier(name),
        }) => {
            /* check for keywords */
            if is_keyword(name) {
                Ok(None)
            } else {
                lexer.advance_mut()?;
                Ok(Some(Identifier::new(name, location)))
            }
        }
        Some(_) => Ok(None),
    }
}

fn parse_tuple_predicate<'a>(
    location: Location<'a>,
    lexer: &mut Lexer<'a>,
) -> ParseResult<'a, Option<Predicate<'a>>> {
    let mut predicates: Vec<Box<Predicate>> = Vec::new();
    loop {
        match parse_predicate(lexer)? {
            Some(predicate) => {
                if lexer.peek_matches(Lexeme::Comma) {
                    println!("AA {:?}", predicate);
                    predicates.push(Box::new(predicate));
                    lexer.advance_mut()?;
                } else {
                    lexer.chomp(Lexeme::RParen)?;
                    if predicates.len() == 0 {
                        return Ok(Some(predicate));
                    } else if predicates.len() >= 1 {
                        predicates.push(Box::new(predicate));
                    }
                    return Ok(Some(Predicate::Tuple {
                        location,
                        dims: predicates,
                    }));
                };
            }
            None => {
                break;
            }
        }
    }
    lexer.chomp(Lexeme::RParen)?;
    Ok(Some(Predicate::Tuple {
        location,
        dims: predicates,
    }))
}

fn parse_predicate<'a>(lexer: &'a mut Lexer) -> ParseResult<'a, Option<Predicate<'a>>> {
    match lexer.peek() {
        Some(token) => match token.lexeme {
            Lexeme::Signed(value) => {
                lexer.advance_mut()?;
                Ok(Some(Predicate::Integer {
                    location: token.location,
                    value,
                }))
            }
            Lexeme::QuotedString(value) => {
                lexer.advance_mut()?;
                Ok(Some(Predicate::String {
                    location: token.location,
                    value: value.to_string(),
                }))
            }
            Lexeme::Identifier(name) => {
                // Ctor
                if name.chars().nth(0).unwrap().is_uppercase() {
                    let ctor_id = Identifier::new(name, token.location);
                    lexer.advance_mut()?;
                    let predicates = parse_predicates(lexer)?;
                    Ok(Some(Predicate::Ctor {
                        ctor_id,
                        dims: predicates,
                    }))
                } else {
                    let loc = lexer.location.clone();
                    lexer.advance_mut()?;
                    Ok(Some(Predicate::Irrefutable(Identifier::new(name, loc))))
                }
            }
            Lexeme::LParen => {
                lexer.advance_mut()?;
                parse_tuple_predicate(token.location, lexer)
            }
            _ => Ok(None),
        },
        None => {
            return Err(ParseError::error(
                lexer.location,
                "missing token where a predicate was expected?",
            ))
        }
    }
}

fn parse_predicates<'a>(lexer: &mut Lexer<'a>) -> ParseResult<'a, Vec<Box<Predicate<'a>>>> {
    let mut predicates = Vec::new();
    loop {
        match parse_predicate(lexer)? {
            None => return Ok(predicates),
            Some(predicate) => {
                println!(
                    "{}: found predicate {:?}",
                    predicate.get_location(),
                    predicate
                );
                predicates.push(Box::new(predicate));
            }
        }
    }
}

fn parse_identifier<'a>(lexer: &mut Lexer<'a>) -> ParseResult<'a, Identifier<'a>> {
    match lexer.peek() {
        Some(Token {
            location,
            lexeme: Lexeme::Identifier(name),
        }) => {
            lexer.advance_mut()?;
            Ok(Identifier::new(name, location))
        }
        _ => Err(ParseError::error(
            lexer.location,
            "expected an identifier here",
        )),
    }
}

fn parse_match_expr<'a>(
    _location: Location<'a>,
    lexer: &mut Lexer<'a>,
) -> ParseResult<'a, Option<Box<Expr<'a>>>> {
    let _binding_value = parse_callsite(lexer)?;
    loop {
        lexer.skip_semicolon()?;
        match parse_predicate(lexer)? {
            Some(_predicate) => {
                lexer.chomp(Lexeme::Operator("=>"))?;
                break;
            }
            None => {
                break;
            }
        }
    }

    Ok(None)
}

fn parse_let_expr<'a>(
    location: Location<'a>,
    lexer: &mut Lexer<'a>,
) -> ParseResult<'a, Option<Box<Expr<'a>>>> {
    let binding_id = parse_identifier(&mut lexer)?;
    lexer.chomp(Lexeme::Operator("="))?;
    let binding_value = parse_callsite(&mut lexer)?;
    lexer.chomp(Lexeme::Identifier("in"))?;
    let in_body = parse_callsite(&mut lexer)?;
    Ok(Some(
        Expr::Let {
            location: location,
            binding: binding_id,
            value: binding_value.into(),
            body: in_body.into(),
        }
        .into(),
    ))
}

fn parse_callsite_term<'a>(lexer: &mut Lexer<'a>) -> ParseResult<'a, Option<Box<Expr<'a>>>> {
    match lexer.peek() {
        None => {
            println!("AABAB");
            lexer.advance_mut()?;
            Ok(None)
        }
        Some(Token { location, lexeme }) => match lexeme {
            // A symbol reference.
            Lexeme::Identifier(name) => {
                println!("KKJDKF");
                if name == "let" {
                    let loc = lexer.advance_mut()?;
                    parse_let_expr(loc, lexer)
                } else if name == "match" {
                    let location = lexer.advance_mut()?;
                    parse_match_expr(location, lexer)
                } else if is_keyword(name) {
                    println!("FIXME: not impl keyword? '{}'", name);
                    Ok(None)
                } else {
                    lexer.advance()?;
                    Ok(Some(
                        Expr::Symbol {
                            id: Identifier::new(name, location),
                        }
                        .into(),
                    ))
                }
            }
            Lexeme::Semicolon => {
                lexer.advance_mut()?;
                Ok(None)
            }
            Lexeme::Operator("=") => Ok(None),
            Lexeme::LParen => {
                lexer.advance()?;
                let expr = parse_callsite(&mut lexer)?;
                lexer.chomp(Lexeme::RParen)?;
                Ok(Some(expr.into()))
            }
            Lexeme::RParen => Ok(None),
            Lexeme::Operator(name) => {
                lexer.advance()?;
                Ok(Some(
                    Expr::Symbol {
                        id: Identifier::new(name, location),
                    }
                    .into(),
                ))
            }
            Lexeme::QuotedString(value) => {
                lexer.advance()?;
                Ok(Some(
                    Expr::LiteralString {
                        location,
                        value: value.into(),
                    }
                    .into(),
                ))
            }
            Lexeme::Signed(value) => {
                lexer.advance()?;
                Ok(Some(Expr::LiteralInteger { location, value }.into()))
            }
            Lexeme::Float(value) => {
                lexer.advance()?;
                Ok(Some(Expr::LiteralFloat { location, value }.into()))
            }
            lexeme => {
                eprintln!("{}: ran into {:?}", location, lexeme);
                Err(ParseError::not_impl(location))
            }
        },
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

fn parse_callsite<'a>(lexer: &mut Lexer<'a>) -> ParseResult<'a, Expr<'a>> {
    lexer.skip_semicolon()?;
    let new_lexer = lexer.clone();
    let maybe_function = parse_callsite_term(&mut new_lexer)?;
    *lexer = new_lexer;

    match maybe_function {
        Some(function) => match parse_many(parse_callsite_term, lexer)? {
            callsite_terms => {
                if callsite_terms.len() == 0 {
                    Ok(*function)
                } else {
                    Ok(Expr::Callsite {
                        function,
                        arguments: callsite_terms,
                    })
                }
            }
        },
        None => Err(ParseError::error(
            lexer.location,
            "missing function callsite expression",
        )),
    }
}

pub fn parse_decl<'a>(lexer: &mut Lexer<'a>) -> ParseResult<'a, Option<Decl<'a>>> {
    let id = match maybe_id(lexer)? {
        Some(id) => id,
        None => return Ok(None),
    };
    let predicates = parse_predicates(&mut lexer)?;
    println!("got done with predicates for {}", &id.name);
    lexer.chomp(Lexeme::Operator("="))?;
    let expr = parse_callsite(&mut lexer)?;
    println!("{}: Found callsite {:?}", expr.get_location(), expr);
    Ok(Some({
        let decl = Decl {
            id,
            predicates,
            body: expr,
        };
        println!("{}: found decl {:?}", decl.get_location(), decl);
        decl
    }))
}

pub fn parse_many<'a, T, P>(parser: P, lexer: &mut Lexer<'a>) -> Result<Vec<T>, ParseError<'a>>
where
    T: 'a + std::fmt::Debug + HasLocation<'a>,
    P: 'a + Fn(&mut Lexer<'a>) -> Result<Option<T>, ParseError<'a>>,
{
    let mut objects = Vec::new();
    loop {
        let new_lexer = lexer.clone();
        match parser(&mut new_lexer)? {
            Some(object) => {
                *lexer = new_lexer;
                // let loc = object.get_location();
                // println!("{}: info: found a thing! {:?}", loc, object);
                objects.push(object);
            }
            None => return Ok(objects),
        }
    }
}
