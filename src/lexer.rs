use crate::error::ParseError;
use crate::location::Location;
use crate::token::Token;

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
    pub fn peek(self) -> (Option<Token<'a>>, Self) {
        match self.state {
            LexState::Started => (None, self),
            LexState::Read(ref token) => {
                println!("{}: lexing  {:?}", token.location, token);
                (Some(token.clone()), self)
            }
            LexState::EOF => (None, self),
        }
    }

    pub fn peek_matches(&self, expect_lexeme: Lexeme<'a>) -> bool {
        match self.state {
            LexState::Started => false,
            LexState::Read(ref token) => token.lexeme == expect_lexeme,
            LexState::EOF => false,
        }
    }

    pub fn chomp(self, expect_lexeme: Lexeme<'a>) -> Result<Self, ParseError<'a>> {
        match self.state {
            LexState::Started => Err(ParseError::error(self.location, "lexer was not started!")),
            LexState::Read(ref token) => {
                if token.lexeme == expect_lexeme {
                    Ok(self.advance())
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

    pub fn advance(mut self) -> Self {
        if self.state == LexState::EOF {
            return self;
        }

        if self.contents.len() == 0 {
            self.state = LexState::EOF;
            return self;
        }

        enum LS {
            Start,
            Identifier,
            Digits,
            Operator,
            Minus,
            QuotedString,
        }
        let mut ls = LS::Start;
        let mut start_location = self.location.clone();
        let mut count = 0;
        let mut lexeme_start = self.contents;
        let mut lexeme_start_index = 0;
        let mut ch_iter = self.contents.chars();
        loop {
            let ch: char = ch_iter.next().unwrap_or('\0');

            match ls {
                LS::Start => {
                    self.update_loc(ch);
                    if ch == '\0' {
                        self.state = LexState::EOF;
                        return self;
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
                        return self._advance(ch, count, Lexeme::LParen);
                    } else if ch == ')' {
                        return self._advance(ch, count, Lexeme::RParen);
                    } else if ch == '{' {
                        return self._advance(ch, count, Lexeme::LCurly);
                    } else if ch == '}' {
                        return self._advance(ch, count, Lexeme::RCurly);
                    } else if ch == '[' {
                        return self._advance(ch, count, Lexeme::LSquare);
                    } else if ch == ']' {
                        return self._advance(ch, count, Lexeme::RSquare);
                    } else if ch == ';' {
                        return self._advance(ch, count, Lexeme::Semicolon);
                    } else if ch == ',' {
                        return self._advance(ch, count, Lexeme::Comma);
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
                        return self;
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
                        return self;
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
                        return self;
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
                        return self;
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
                        return self;
                    }
                }
            }
        }
    }

    fn _advance(mut self, ch: char, mut count: usize, lexeme: Lexeme<'a>) -> Self {
        count += ch.len_utf8();
        self.contents = &self.contents[count..];
        self.state = LexState::Read(Token {
            location: self.location.clone(),
            lexeme: lexeme,
        });
        self
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
