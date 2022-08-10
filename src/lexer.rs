use std::fmt;
use std::fmt::Formatter;
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Lexeme<'a> {
    Unsigned(u64),
    Float(f64),
    Identifier(&'a str),
    QuotedString(&'a str),
    Operator(&'a str),
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    Colon,
    Semicolon,
    Dot,
    Ampersand,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Location<'a> {
    filename: &'a str,
    line: i32,
    col: i32,
}

impl<'a> std::fmt::Display for Location<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.col)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Token<'a> {
    pub location: Location<'a>,
    pub lexeme: Lexeme<'a>,
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.lexeme)
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
    location: Location<'a>,
    state: LexState<'a>,
}

impl<'a> Lexer<'a> {
    pub fn peek(self) -> (Option<Token<'a>>, Self) {
        match self.state {
            LexState::Started => (None, self),
            LexState::Read(ref token) => (Some(token.clone()), self),
            LexState::EOF => (None, self),
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
        }
        let mut ls = LS::Start;
        let mut start_location = self.location.clone();
        let mut count = 0;
        let mut lexeme_start = self.contents;
        let mut lexeme_start_index = 0;
        let mut ch_iter = self.contents.chars();
        loop {
            let ch: char = ch_iter.next().unwrap_or('\0');
            self.update_loc(ch);

            match ls {
                LS::Start => {
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
                    } else if ch == ':' {
                        return self._advance(ch, count, Lexeme::Colon);
                    } else if ch == ';' {
                        return self._advance(ch, count, Lexeme::Semicolon);
                    } else if ch == '&' {
                        return self._advance(ch, count, Lexeme::Ampersand);
                    } else if ch == '.' {
                        return self._advance(ch, count, Lexeme::Dot);
                    } else if ch == '(' {
                        return self._advance(ch, count, Lexeme::LParen);
                    } else if ch == ')' {
                        return self._advance(ch, count, Lexeme::RParen);
                    } else if ch == '[' {
                        return self._advance(ch, count, Lexeme::LSquare);
                    } else if ch == ']' {
                        return self._advance(ch, count, Lexeme::RSquare);
                    } else if ch == '{' {
                        return self._advance(ch, count, Lexeme::LCurly);
                    } else if ch == '}' {
                        return self._advance(ch, count, Lexeme::RCurly);
                    } else {
                        assert!(false, "could not figure out what do do with '{ch}'!");
                    }

                    count += ch.len_utf8();
                }
                LS::Identifier => {
                    if ch == '_' || ch.is_alphanumeric() {
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
                LS::Digits => {
                    if ch.is_digit(10) {
                        count += ch.len_utf8();
                    } else {
                        self.contents = &self.contents[count..];
                        self.state = LexState::Read(Token {
                            location: start_location,
                            lexeme: Lexeme::Unsigned(
                                lexeme_start[..count - lexeme_start_index]
                                    .parse::<u64>()
                                    .unwrap(),
                            ),
                        });
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

    pub fn new<T>(filename: T, input: T) -> Self
    where
        T: Into<&'a str>,
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
            self.location.col += 1;
        }
    }
}
