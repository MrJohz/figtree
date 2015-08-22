use super::lexer::{Lexer, LexToken};
use std::io::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseEvent {
    BeginFile,
    EndOfFile,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidState,
}

pub struct Parser<R: Read> {
    current_state: Option<ParseEvent>,
    next_token: Option<LexToken>,
    pub last_error: Option<ParseError>,
    lexer: Lexer<R>,
}

impl<R: Read> Parser<R> {
    pub fn parse(lexer: Lexer<R>) -> Self {
        Parser {
            current_state: None,
            next_token: None,
            last_error: None,
            lexer: lexer,
        }
    }

    fn yield_state(&mut self, state: ParseEvent) -> Option<ParseEvent> {
        self.current_state = Some(state);
        self.next_token = self.lexer.next();
        Some(state)
    }

    fn yield_error(&mut self, error: ParseError) -> Option<ParseEvent> {
        self.last_error = Some(error);
        None
    }
}

impl<R: Read> Iterator for Parser<R> {
    type Item = ParseEvent;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_state {
            None => self.yield_state(ParseEvent::BeginFile),
            Some(ParseEvent::BeginFile) => {
                match self.next_token {
                    None => self.yield_state(ParseEvent::EndOfFile),
                    Some(_) => self.yield_error(ParseError::InvalidState),
                }
            }
            Some(ParseEvent::EndOfFile) => {
                match self.next_token {
                    None => None,
                    Some(_) => self.yield_error(ParseError::InvalidState),
                }
            }
        }


    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lexer::Lexer;
    use std::io::Cursor;

    #[test]
    fn handle_empty_file() {
        let file = Cursor::new("".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next(), Some(ParseEvent::BeginFile));
        assert_eq!(parser.next(), Some(ParseEvent::EndOfFile));
        assert_eq!(parser.next(), None);
        assert_eq!(parser.last_error, None);
    }
}
