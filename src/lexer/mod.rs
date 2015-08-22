extern crate regex;
mod lexutils;

use self::lexutils::TokenCollection;
use std::io::prelude::*;
use std::io::BufReader;
use std::io;

pub enum LexError {
    IOError(io::Error)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Position { pub line: usize, pub pos: usize, }
impl Position {
    pub fn new() -> Self {
        Position { line: 0, pos: 0 }
    }
    pub fn new_line(&mut self) {
        self.line += 1;
        self.pos = 0;
    }
    pub fn push(&mut self, amt: usize) {
        self.pos += amt;
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    OpenBrace, CloseBrace,
    OpenBracket, CloseBracket,
    Comma, Colon,
    Identifier(String),
    StringLit(String),
    IntegerLit(i64),
    FloatLit(f64),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pattern: regex::Regex,
}

impl Token {
    fn new(re: &str) -> Result<Self, regex::Error> {
        Ok(Token {
            pattern: try!(regex::Regex::new(&("^(?:".to_string() + re + ")"))),
        })
    }

    fn get_match(&self, inp: &str) -> Option<usize> {
        if let Some((_, end)) = self.pattern.find(inp) {
            Some(end)
        } else {
            None
        }
    }
}

pub struct Lexer<R: Read> {
    filestream: BufReader<R>,
    buffer: String,
    position: Position,
    ignore: Vec<Token>,
    tokens: TokenCollection,
    errors: Vec<LexError>,
    read_to_end: bool,
}

impl<R: Read> Lexer<R> {

    pub fn lex(file: R) -> Self {
        Lexer {
            filestream: BufReader::new(file),
            buffer: String::new(),
            position: Position::new(),
            errors: Vec::new(),
            read_to_end: false,
            ignore: vec![
                Token::new(r"\s*").unwrap()
            ],
            tokens: TokenCollection::new(),
        }
    }

    fn read_buffer(&mut self, len: usize) -> String {
        let mut position = self.position.clone();
        let response = self.buffer.chars().take(len).collect();
        self.buffer = self.buffer
            .chars()
            .skip(len)
            .map(|c| {
                if c == '\n' {
                    position.new_line();
                } else {
                    position.push(1);
                }
                c
            } )
            .collect();
        self.position = position;
        return response;
    }
    fn can_read(&self, len: usize) -> bool {
        return len == self.buffer.len() && !self.read_to_end;
    }
}

impl<R: Read> Iterator for Lexer<R> {
    type Item = TokenKind;

    fn next(&mut self) -> Option<Self::Item> {
        'mainloop: loop {
            match self.filestream.read_line(&mut self.buffer) {
                Ok(size) => {
                    if size == 0 {
                        self.read_to_end = true;
                        // read to end of file - but have we finished reading the buffer?
                        if self.buffer.len() == 0 {
                            return None;
                        }
                    }
                },
                Err(err) => {
                    self.errors.push(LexError::IOError(err));
                    return None;
                }
            }

            self.position.new_line();

            for ign in &self.ignore {
                if let Some(len) = ign.get_match(&self.buffer) {
                    if self.can_read(len) { continue 'mainloop; }
                    self.buffer = self.buffer.chars().skip(len).collect();
                }
            }

            if let Some(len) = self.tokens.open_brace.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.read_buffer(len);
                return Some(TokenKind::OpenBrace);
            } else if let Some(len) = self.tokens.close_brace.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.read_buffer(len);
                return Some(TokenKind::CloseBrace);
            } else if let Some(len) = self.tokens.open_bracket.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.read_buffer(len);
                return Some(TokenKind::OpenBracket);
            } else if let Some(len) = self.tokens.close_bracket.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.read_buffer(len);
                return Some(TokenKind::CloseBracket);
            } else if let Some(len) = self.tokens.comma.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.read_buffer(len);
                return Some(TokenKind::Comma);
            } else if let Some(len) = self.tokens.colon.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.read_buffer(len);
                return Some(TokenKind::Colon);
            } else if let Some(len) = self.tokens.identifier.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                return Some(TokenKind::Identifier(self.read_buffer(len)));
            } else if let Some(len) = self.tokens.stringlit.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                return Some(TokenKind::StringLit(
                    lexutils::parse_string(self.read_buffer(len))));
            } else if let Some(len) = self.tokens.floatlit.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                return Some(TokenKind::FloatLit(
                    lexutils::parse_float(self.read_buffer(len))));
            } else if let Some(len) = self.tokens.integerlit.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                return Some(TokenKind::IntegerLit(
                    lexutils::parse_integer(self.read_buffer(len))));
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    mod token {
        use super::*;
        extern crate regex;

        #[test]
        fn test_construction() {
            let tok = Token::new("xyz").unwrap();
            assert_eq!(tok.pattern, regex::Regex::new("^(?:xyz)").unwrap());
        }

        #[test]
        fn test_matching() {
            let tok = Token::new("xyz").unwrap();
            assert_eq!(tok.get_match("xyzdjsd"), Some(3));
            assert_eq!(tok.get_match("abcdjsd"), None);
        }
    }

    mod lexer {
        use super::*;
        use std::io::Cursor;

        #[test]
        fn test_lex_iterable() {
            let file = Cursor::new("hello {}".as_bytes());
            let mut iter = Lexer::lex(file);
            assert_eq!(iter.next(), Some(TokenKind::Identifier("hello".to_string())));
            assert_eq!(iter.next(), Some(TokenKind::OpenBrace));
            assert_eq!(iter.next(), Some(TokenKind::CloseBrace));
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn lex_all_tokens() {
            let file = Cursor::new("ident {}[],: 'string' 54 3.5e5 false".as_bytes());
            let mut iter = Lexer::lex(file);
            assert_eq!(iter.next(), Some(TokenKind::Identifier("ident".to_string())));
            assert_eq!(iter.next(), Some(TokenKind::OpenBrace));
            assert_eq!(iter.next(), Some(TokenKind::CloseBrace));
            assert_eq!(iter.next(), Some(TokenKind::OpenBracket));
            assert_eq!(iter.next(), Some(TokenKind::CloseBracket));
            assert_eq!(iter.next(), Some(TokenKind::Comma));
            assert_eq!(iter.next(), Some(TokenKind::Colon));
            assert_eq!(iter.next(), Some(TokenKind::StringLit("string".to_string())));
            assert_eq!(iter.next(), Some(TokenKind::IntegerLit(54)));
            assert_eq!(iter.next(), Some(TokenKind::FloatLit(3.5e5_f64)));
            assert_eq!(iter.next(), Some(TokenKind::Identifier("false".to_string())));
            assert_eq!(iter.next(), None);
        }
    }
}
