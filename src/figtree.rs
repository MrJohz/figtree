use std::io::Cursor;
use std::io::prelude::*;
use super::parser::{Parser, ParseEvent, ParseError};
use super::lexer::Lexer;
use super::position::Position;

use super::types::*;

pub struct Figtree {
    parser: Parser,
}

impl Figtree {
    pub fn new<T: Read>(input: T) -> Self {
        Figtree {
            parser: Parser::parse(Lexer::lex(input))
        }
    }

    pub fn from_string(input: &str) -> Figtree {
        Figtree::new(Cursor::new(input.as_bytes()))
    }

    pub fn parse(&mut self) -> Result<Document, (ParseError, Position)> {
        let mut doc = Document::new();
        match self.parser.next() {
            Some(Ok((ParseEvent::FileStart, _))) => {
                if let Some(err) = self.parse_file(&mut doc) {
                    return Err(err);
                }
            }
            Some(Ok(_)) | None =>
                unreachable!("ParseEvent occurred that cannot happen at this time."),
            Some(Err(error)) =>
                return Err(error),
        }
        Ok(doc)
    }

    fn parse_file(&mut self, doc: &mut Document) -> Option<(ParseError, Position)> {
        loop {
            match self.parser.next() {
                Some(Ok((ParseEvent::NodeStart(name), _))) => {
                    if let Some(err) = self.parse_node(doc.new_node(name)) {
                        return Some(err);
                    }
                },
                Some(Ok((ParseEvent::FileEnd, _))) => {
                    return None;
                },
                Some(Ok(ev)) =>
                    unreachable!("ParseEvent {:?} occured that cannot happen at this time.", ev),
                Some(Err(error)) => { return Some(error) },
                None =>
                    unreachable!("EOF occured that cannot happen at this time."),
            }
        }
    }

    fn parse_node(&mut self, doc: &mut Node) -> Option<(ParseError, Position)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Figtree;
    use std::io::Cursor;

    #[test]
    fn construct_empty_file() {
        let mut figgy = Figtree::from_string("");
        let config = figgy.parse().unwrap();
        assert_eq!(config.nodes.len(), 0);
    }
}
