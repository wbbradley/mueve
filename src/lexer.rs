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

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Token<'a> {
    location: Location<'a>,
    lexeme: Lexeme<'a>,
}

#[allow(dead_code)]
pub struct Lexer<'a> {
    contents: &'a str,
    location: Location<'a>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.contents.len() == 0 {
            return None;
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
                        return None;
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
                        return Some(Token {
                            location: start_location,
                            lexeme: Lexeme::Identifier(&lexeme_start[..count - lexeme_start_index]),
                        });
                    }
                }
                LS::Digits => {
                    if ch.is_digit(10) {
                        count += ch.len_utf8();
                    } else {
                        self.contents = &self.contents[count..];
                        return Some(Token {
                            location: start_location,
                            lexeme: Lexeme::Unsigned(
                                lexeme_start[..count - lexeme_start_index]
                                    .parse::<u64>()
                                    .unwrap(),
                            ),
                        });
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.contents.len()))
    }
}

impl<'a> Lexer<'a> {
    fn _advance(&mut self, ch: char, mut count: usize, lexeme: Lexeme<'a>) -> Option<Token<'a>> {
        count += ch.len_utf8();
        self.contents = &self.contents[count..];
        return Some(Token {
            location: self.location.clone(),
            lexeme: lexeme,
        });
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
