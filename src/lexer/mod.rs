extern crate regex;
mod lexutils;

use self::lexutils::TokenCollection;
use std::io::prelude::*;
use std::io::BufReader;
use std::io;

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
    IOError(io::Error)
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
    errored: bool,
    read_to_end: bool,
}

impl<R: Read> Lexer<R> {

    pub fn lex(file: R) -> Self {
        Lexer {
            filestream: BufReader::new(file),
            buffer: String::new(),
            position: Position::new(),
            errored: false,
            read_to_end: false,
            ignore: vec![
                Token::new(r"\s*").unwrap()
            ],
            tokens: TokenCollection::new(),
        }
    }

    fn consume_buffer(&mut self, len: usize) -> String {
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
    type Item = Result<LexToken, LexError>;

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
                    return Some(Err(LexError::IOError(err)));
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
                self.consume_buffer(len);
                return Some(Ok(LexToken::OpenBrace));
            } else if let Some(len) = self.tokens.close_brace.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.consume_buffer(len);
                return Some(Ok(LexToken::CloseBrace));
            } else if let Some(len) = self.tokens.open_bracket.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.consume_buffer(len);
                return Some(Ok(LexToken::OpenBracket));
            } else if let Some(len) = self.tokens.close_bracket.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.consume_buffer(len);
                return Some(Ok(LexToken::CloseBracket));
            } else if let Some(len) = self.tokens.comma.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.consume_buffer(len);
                return Some(Ok(LexToken::Comma));
            } else if let Some(len) = self.tokens.colon.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                self.consume_buffer(len);
                return Some(Ok(LexToken::Colon));
            } else if let Some(len) = self.tokens.identifier.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                return Some(Ok(LexToken::Identifier(self.consume_buffer(len))));
            } else if let Some(len) = self.tokens.stringlit.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                return Some(Ok(LexToken::StringLit(
                    lexutils::parse_string(self.consume_buffer(len)))));
            } else if let Some(len) = self.tokens.floatlit.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                return Some(Ok(LexToken::FloatLit(
                    lexutils::parse_float(self.consume_buffer(len)))));
            } else if let Some(len) = self.tokens.integerlit.get_match(&self.buffer) {
                if self.can_read(len) { continue 'mainloop; }
                return Some(Ok(LexToken::IntegerLit(
                    lexutils::parse_integer(self.consume_buffer(len)))));
            } else {
                if self.read_to_end {
                    return None;
                } else {
                    continue 'mainloop;
                }
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
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::Identifier("hello".to_string()));
            assert_eq!(iter.next().unwrap().unwrap(), LexToken::OpenBrace);
            assert_eq!(iter.next().unwrap().unwrap(), LexToken::CloseBrace);
            assert!(iter.next().is_none());
        }

        #[test] #[ignore] // Comments not yet implemented
        fn lex_ignored_tokens() {
            let file = Cursor::new(
                "
                token1
                // hello
                token2
                /* one line */
                token3
                /* multiple
                token4
                 * lines */
                 token5".as_bytes());
            let mut iter = Lexer::lex(file);
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::Identifier("token1".to_string()));
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::Identifier("token2".to_string()));
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::Identifier("token3".to_string()));
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::Identifier("token5".to_string()));
            assert!(iter.next().is_none());
        }

        #[test]
        fn lex_all_tokens() {
            let file = Cursor::new("ident {}[],: 'string' 54 3.5e5 false".as_bytes());
            let mut iter = Lexer::lex(file);
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::Identifier("ident".to_string()));
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::OpenBrace);
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::CloseBrace);
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::OpenBracket);
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::CloseBracket);
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::Comma);
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::Colon);
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::StringLit("string".to_string()));
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::IntegerLit(54));
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::FloatLit(3.5e5_f64));
            assert_eq!(iter.next().unwrap().unwrap(),
                LexToken::Identifier("false".to_string()));
            assert!(iter.next().is_none());
        }
    }
}
