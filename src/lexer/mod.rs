use std::io::prelude::*;
use std::io;
use super::utils::CharReader;

const DECIMAL_NUMERIC: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_'];
const OCTAL_NUMERIC: [char; 9] = ['0', '1', '2', '3', '4', '5', '6', '7', '_'];
const BINARY_NUMERIC: [char; 3] = ['0', '1', '_'];
const HEXA_NUMERIC: [char; 22] =
    ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_',
     'a', 'b', 'c', 'e', 'f', 'A', 'B', 'C', 'D', 'E', 'F'];

type LexReader<R: Read> = CharReader<io::BufReader<R>>;
type LexResult = Result<LexToken, LexError>;

#[derive(Debug, PartialEq, Clone)]
pub enum LexToken {
    OpenBrace, CloseBrace,
    OpenBracket, CloseBracket,
    Comma, Colon,
    Identifier(String),
    StringLit(String),
    IntegerLit(i64),
    FloatLit(f64),
}

#[derive(Debug)]
pub enum LexError {
    IOError(io::Error),
    UnrecognisedCharError(char),
}

pub struct Lexer<R: Read> {
    input: LexReader<R>,
    stored_next: Option<char>,
}

impl<R: Read> Lexer<R> {
    pub fn parse(reader: R) -> Self {
        Lexer {
            input: CharReader::new(io::BufReader::new(reader)),
            stored_next: None,
        }
    }

    pub fn pop_next(&mut self) -> Option<char> {
        if let Some(next) = self.stored_next {
            self.stored_next = None;
            Some(next)
        } else {
            self.input.next()
        }
    }

    pub fn ret_next(&mut self, returned: char) {
        self.stored_next = Some(returned);
    }

    pub fn parse_ident(&mut self, next_char: char) -> Option<LexResult> {
        let mut ident = String::new();
        ident.push(next_char);

        while let Some(next_char) = self.pop_next() {
            if next_char.is_alphanumeric() {
                ident.push(next_char);
            } else {
                self.ret_next(next_char);
                break;
            }
        }

        Some(Ok(LexToken::Identifier(ident)))
    }

    pub fn parse_int(&mut self, base: u32) -> Option<LexResult> {
        let mut buffer = String::new();

        while let Some(next_char) = self.pop_next() {
            if next_char.is_digit(base) || next_char == '_' {
                buffer.push(next_char);
            } else {
                self.ret_next(next_char);
                break;
            }
        }

        Some(Ok(LexToken::IntegerLit(i64::from_str_radix(&buffer, base).unwrap())))
    }

    pub fn parse_numeric(&mut self, next_char: char) -> Option<LexResult> {
        if let Some(after) = self.pop_next() {
            if next_char == '0' && ['d', 'D'].contains(&after) {
                self.pop_next(); // discard
                self.parse_int(10)
            } else if next_char == '0' && ['x', 'X'].contains(&after)  {
                self.pop_next(); // discard
                self.parse_int(16)
            } else if next_char == '0' && ['o', 'O'].contains(&after) {
                self.pop_next(); // discard
                self.parse_int(8)
            } else if next_char == '0' && ['b', 'B'].contains(&after) {
                self.pop_next(); // discard
                self.parse_int(2)
            } else {
                // TODO: parse non 0x* numbers
                self.ret_next(after);
                None
            }
        } else {
            Some(Ok(LexToken::IntegerLit(next_char.to_digit(10).unwrap() as i64)))
        }
    }
}

impl<R: Read> Iterator for Lexer<R> {
    type Item = Result<LexToken, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let option_char = self.pop_next();
        while option_char.is_some() && option_char.unwrap().is_whitespace() {
            let option_char = self.pop_next();
        }

        if let Some(next_char) = option_char {
            if next_char.is_alphabetic() {
                self.parse_ident(next_char)
            } else if next_char.is_digit(10) || ['+', '-'].contains(&next_char) {
                self.parse_numeric(next_char)
            } else {
                None
            }
        } else {
            None
        }
    }
}
