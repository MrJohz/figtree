use std::io::Cursor;
use std::io::prelude::*;
use super::parser::{Parser, ParseEvent, ParseError};
use super::lexer::Lexer;
use super::position::Position;

use super::types::*;

pub struct Figtree<T: Read> {
    parser: Parser<T>,
}

impl<T: Read> Figtree<T> {
    pub fn new(input: T) -> Self {
        Figtree {
            parser: Parser::parse(Lexer::lex(input))
        }
    }

    pub fn from_string(input: &str) -> Figtree<Cursor<&[u8]>> {
        Figtree::new(Cursor::new(input.as_bytes()))
    }

    pub fn parse(&mut self) -> Result<Document, (ParseError, Position)> {
        let mut doc = Document::new();
        match self.parser.next() {
            Some(Ok((ParseEvent::FileStart, _))) =>
                self.parse_file(&mut doc),
            Some(Ok(_)) | None =>
                panic!("ParseEvent occurred that cannot happen at this time."),
            Some(Err(error)) =>
                return Err(error),
        }
        Ok(doc)
    }

    fn parse_file(&mut self, doc: &mut Document) {
        // TODO: implement!
    }
}
