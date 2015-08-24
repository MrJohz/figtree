use super::lexer::{Lexer, LexToken, LexError};
use std::io::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseEvent {
    ERR,  // internal use only
    BeginFile,
    EndOfFile,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidState,
    LexError(LexError),
}

pub type ParseResult = Result<ParseEvent, ParseError>;

pub struct Parser<R: Read> {
    current_state: Option<ParseEvent>,
    next_token: Option<LexToken>,
    lexer: Lexer<R>,
}

impl<R: Read> Parser<R> {
    pub fn parse(lexer: Lexer<R>) -> Self {
        Parser {
            current_state: None,
            next_token: None,
            lexer: lexer,
        }
    }

    fn yield_state(&mut self, state: ParseEvent) -> Option<ParseResult> {
        self.current_state = Some(state);
        if let Some(token) = self.lexer.next() {
            match token {
                Ok(token) => { self.next_token = Some(token); },
                Err(err) => { return Some(Err(ParseError::LexError(err))); }
            }
        } else {
            self.next_token = None;
        }
        Some(Ok(self.current_state.unwrap()))
    }
    fn yield_error(&mut self, error: ParseError) -> Option<ParseResult> {

        self.current_state = Some(ParseEvent::ERR);
        Some(Err(error))
    }
}

impl<R: Read> Iterator for Parser<R> {
    type Item = Result<ParseEvent, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_state {
            None => self.yield_state(ParseEvent::BeginFile),
            Some(ParseEvent::BeginFile) => {
                match self.next_token {
                    None => self.yield_state(ParseEvent::EndOfFile),
                    Some(_) => self.yield_error(ParseError::InvalidState),
                }
            },
            Some(ParseEvent::EndOfFile) => None,
            Some(ParseEvent::ERR) => None,
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
        assert_eq!(parser.next().unwrap().unwrap(), ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap(), ParseEvent::EndOfFile);
        assert!(parser.next().is_none());
    }
}
