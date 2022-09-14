use crate::error::{ParseError, ParseResult};
use crate::location::Location;
use crate::token::Token;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Lexeme<'a> {
    Signed(i64),
    Float(f64),
    Identifier(&'a str),
    QuotedString(&'a str),
    Operator(&'a str),
    Semicolon,
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    Comma,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BracketType {
    Paren,
    Square,
    Curly,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Nesting<'a> {
    location: Location<'a>,
    bt: BracketType,
    next: Option<Rc<Nesting<'a>>>,
}

fn push_nested_bracket<'a>(
    location: Location<'a>,
    bt: BracketType,
    next: Option<Rc<Nesting<'a>>>,
) -> Option<Rc<Nesting<'a>>> {
    Some(Rc::new(Nesting { location, bt, next }))
}

fn pop_nested_bracket<'a>(
    nesting: Option<Rc<Nesting<'a>>>,
    location: Location<'a>,
    bt: BracketType,
) -> ParseResult<'a, Option<Rc<Nesting<'a>>>> {
    if let Some(nesting) = nesting {
        match *nesting {
            Nesting {
                location: top_location,
                bt: top_bt,
                ref next,
            } => {
                if bt == top_bt {
                    // Peel off this head node, and return whatever's beneath.
                    Ok(next.clone())
                } else {
                    Err(ParseError::error(
                        location,
                        format!(
                            "encountered a {:?} but expected to close a {:?} from {}",
                            bt, top_bt, top_location
                        ),
                    ))
                }
            }
        }
    } else {
        Err(ParseError::error(
            location,
            format!(
                "encountered a {:?} but we're not inside of any nested syntax",
                bt,
            ),
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexState<'a> {
    Started,
    Read(Token<'a>),
    EOF,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Lexer<'a> {
    contents: &'a str,
    pub location: Location<'a>,
    nesting: Option<Rc<Nesting<'a>>>,
    state: LexState<'a>,
}

fn is_operator_char(ch: char) -> bool {
    return ch == '.'
        || ch == '='
        || ch == '>'
        || ch == '<'
        || ch == '-'
        || ch == '+'
        || ch == '!'
        || ch == '@'
        || ch == ':'
        || ch == '$'
        || ch == '%'
        || ch == '^'
        || ch == '&'
        || ch == '*'
        || ch == '/'
        || ch == '?'
        || ch == '~';
}
impl<'a> Lexer<'a> {
    pub fn skip_semicolon(&mut self) -> ParseResult<'a, ()> {
        while let Some(Token {
            lexeme: Lexeme::Semicolon,
            ..
        }) = self.peek()
        {
            self.advance_mut()?;
        }
        Ok(())
    }

    pub fn peek(&self) -> Option<Token<'a>> {
        match self.state {
            LexState::Started => None,
            LexState::Read(ref token) => {
                println!("{}: lexing  {:?}", token.location, token);
                Some(token.clone())
            }
            LexState::EOF => None,
        }
    }

    pub fn peek_matches(&self, expect_lexeme: Lexeme<'a>) -> bool {
        match self.state {
            LexState::Started => false,
            LexState::Read(ref token) => token.lexeme == expect_lexeme,
            LexState::EOF => false,
        }
    }

    pub fn chomp(&mut self, expect_lexeme: Lexeme<'a>) -> ParseResult<'a, ()> {
        match self.state {
            LexState::Started => Err(ParseError::error(self.location, "lexer was not started!")),
            LexState::Read(ref token) => {
                if token.lexeme == expect_lexeme {
                    self.advance()
                } else {
                    Err(ParseError::unexpected(
                        token.clone(),
                        format!("{:?}", expect_lexeme),
                    ))
                }
            }
            LexState::EOF => Err(ParseError::error(
                self.location,
                format!("hit EOF but expected {:?}", expect_lexeme),
            )),
        }
    }

    pub fn advance(mut self) -> ParseResult<'a, ()> {
        let _ = self.advance_mut()?;
        Ok(())
    }

    pub fn advance_mut(&mut self) -> ParseResult<'a, Location<'a>> {
        let mut start_location = self.location.clone();

        if self.state == LexState::EOF {
            return Ok(start_location);
        } else if self.contents.len() == 0 {
            self.state = LexState::EOF;
            return Ok(start_location);
        }

        // println!("[advance] {:?}", self.state);
        enum LS {
            Start,
            Identifier,
            Digits,
            Operator,
            Minus,
            QuotedString,
        }
        let mut ls = LS::Start;
        let mut count = 0;
        let mut lexeme_start = self.contents;
        let mut lexeme_start_index = 0;
        let mut ch_iter = self.contents.chars();
        loop {
            let ch: char = ch_iter.next().unwrap_or('\0');

            match ls {
                LS::Start => {
                    if ch == '\n' && self.nesting.is_some() {
                        start_location = self.location.clone();
                        let mut count = 1;
                        loop {
                            // Gobble up all whitespace.
                            match ch_iter.next() {
                                Some(ch) => {
                                    if ch.is_whitespace() {
                                        count += 1;
                                    } else {
                                        break;
                                    }

                                    // This is a lexing discontinuity but it achieves the whitespace
                                    // flexibility we want. If a newline occurs outside of a nested structure,
                                    // then it lexes as a semicolon token.
                                    self.update_loc(ch);
                                }
                                None => break,
                            }
                        }

                        self.contents = &self.contents[count..];
                        self.state = LexState::Read(Token {
                            location: start_location,
                            lexeme: Lexeme::Semicolon,
                        });
                        return Ok(start_location);
                    }
                    self.update_loc(ch);
                    let location = self.location;
                    if ch == '\0' {
                        self.state = LexState::EOF;
                        return Ok(start_location);
                    } else if ch.is_whitespace() {
                    } else if ch.is_digit(10) {
                        ls = LS::Digits;
                        lexeme_start_index = count;
                        lexeme_start = &self.contents[count..];
                        start_location = self.location.clone();
                    } else if ch == '_' || ch.is_alphabetic() {
                        ls = LS::Identifier;
                        lexeme_start_index = count;
                        lexeme_start = &self.contents[count..];
                        start_location = self.location.clone();
                    } else if ch == '-' {
                        ls = LS::Minus;
                        lexeme_start_index = count;
                        lexeme_start = &self.contents[count..];
                        start_location = self.location.clone();
                    } else if is_operator_char(ch) {
                        ls = LS::Operator;
                        lexeme_start_index = count;
                        lexeme_start = &self.contents[count..];
                        start_location = self.location.clone();
                    } else if ch == '"' {
                        ls = LS::QuotedString;
                        lexeme_start_index = count;
                        lexeme_start = &self.contents[count..];
                        start_location = self.location.clone();
                    } else if ch == '(' {
                        return self._advance(ch, count, location, Lexeme::LParen);
                    } else if ch == ')' {
                        return self._advance(ch, count, location, Lexeme::RParen);
                    } else if ch == '{' {
                        return self._advance(ch, count, location, Lexeme::LCurly);
                    } else if ch == '}' {
                        return self._advance(ch, count, location, Lexeme::RCurly);
                    } else if ch == '[' {
                        return self._advance(ch, count, location, Lexeme::LSquare);
                    } else if ch == ']' {
                        return self._advance(ch, count, location, Lexeme::RSquare);
                    } else if ch == ';' {
                        return self._advance(ch, count, location, Lexeme::Semicolon);
                    } else if ch == ',' {
                        return self._advance(ch, count, location, Lexeme::Comma);
                    } else {
                        assert!(
                            false,
                            "could not figure out what do do with character ({ch})"
                        );
                    }

                    count += ch.len_utf8();
                }
                LS::Identifier => {
                    if ch == '_' || ch.is_alphanumeric() {
                        self.update_loc(ch);
                        count += ch.len_utf8();
                    } else {
                        self.contents = &self.contents[count..];
                        self.state = LexState::Read(Token {
                            location: start_location,
                            lexeme: Lexeme::Identifier(&lexeme_start[..count - lexeme_start_index]),
                        });
                        return Ok(start_location);
                    }
                }
                LS::Operator => {
                    if is_operator_char(ch) {
                        self.update_loc(ch);
                        count += ch.len_utf8();
                    } else {
                        // println!("{}: info: found a {:?}", &start_location, &lexeme_start[..count - lexeme_start_index]);
                        self.contents = &self.contents[count..];
                        self.state = LexState::Read(Token {
                            location: start_location,
                            lexeme: Lexeme::Operator(&lexeme_start[..count - lexeme_start_index]),
                        });
                        return Ok(start_location);
                    }
                }
                LS::Minus => {
                    if ch.is_digit(10) {
                        self.update_loc(ch);
                        count += ch.len_utf8();
                        ls = LS::Digits;
                    } else if is_operator_char(ch) {
                        self.update_loc(ch);
                        count += ch.len_utf8();
                        ls = LS::Operator;
                    } else {
                        self.contents = &self.contents[count..];
                        self.state = LexState::Read(Token {
                            location: start_location,
                            lexeme: Lexeme::Operator(&lexeme_start[..count - lexeme_start_index]),
                        });
                        return Ok(start_location);
                    }
                }
                LS::Digits => {
                    if ch.is_digit(10) {
                        self.update_loc(ch);
                        count += ch.len_utf8();
                    } else {
                        self.contents = &self.contents[count..];
                        self.state = LexState::Read(Token {
                            location: start_location,
                            lexeme: Lexeme::Signed(
                                lexeme_start[..count - lexeme_start_index]
                                    .parse::<i64>()
                                    .unwrap(),
                            ),
                        });
                        return Ok(start_location);
                    }
                }
                LS::QuotedString => {
                    count += ch.len_utf8();
                    if ch != '"' {
                        self.update_loc(ch);
                    } else {
                        self.contents = &self.contents[count..];
                        self.state = LexState::Read(Token {
                            location: start_location,
                            lexeme: Lexeme::QuotedString(
                                &lexeme_start[..count - lexeme_start_index + 1],
                            ),
                        });
                        println!("lexed {}", &lexeme_start[..count - lexeme_start_index]);
                        return Ok(start_location);
                    }
                }
            }
        }
    }

    fn _advance(
        &mut self,
        ch: char,
        mut count: usize,
        location: Location<'a>,
        lexeme: Lexeme<'a>,
    ) -> ParseResult<'a, Location<'a>> {
        // TODO: make this a stack.
        match lexeme {
            Lexeme::LParen => {
                self.nesting =
                    push_nested_bracket(location, BracketType::Paren, self.nesting.clone());
            }
            Lexeme::RParen => {
                self.nesting =
                    pop_nested_bracket(self.nesting.clone(), location, BracketType::Paren)?;
            }
            Lexeme::LSquare => {
                self.nesting = Some(Rc::new(Nesting {
                    location,
                    bt: BracketType::Square,
                    next: self.nesting.clone(),
                }));
            }
            Lexeme::RSquare => {
                self.nesting =
                    pop_nested_bracket(self.nesting.clone(), location, BracketType::Square)?;
            }
            Lexeme::LCurly => {
                self.nesting = Some(Rc::new(Nesting {
                    location,
                    bt: BracketType::Curly,
                    next: self.nesting.clone(),
                }));
            }
            Lexeme::RCurly => {
                self.nesting =
                    pop_nested_bracket(self.nesting.clone(), location, BracketType::Curly)?;
            }
            _ => (),
        }

        count += ch.len_utf8();
        self.contents = &self.contents[count..];
        self.state = LexState::Read(Token {
            location: self.location,
            lexeme: lexeme,
        });
        Ok(location)
    }

    pub fn new<T, U>(filename: T, input: U) -> Self
    where
        T: 'a + Into<&'a str>,
        U: 'a + Into<&'a str>,
    {
        Lexer {
            contents: input.into(),
            location: Location {
                filename: filename.into(),
                line: 1,
                col: 0,
            },
            state: LexState::Started,
            nesting: None,
        }
    }

    #[inline]
    fn update_loc(&mut self, ch: char) {
        if ch == '\n' {
            self.location.line += 1;
            self.location.col = 0;
        } else {
            // println!("found {}, bumping col", ch);
            self.location.col += 1;
        }
    }
}
