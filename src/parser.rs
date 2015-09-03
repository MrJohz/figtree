use super::lexer::{Lexer, LexToken, LexError};
use super::position::Position;
use std::io::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum ParseEvent {
    BeginFile,
    EndOfFile,
    NodeStart(String),
    NodeEnd,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidState,
    NotYetImplemented,
    LexError(LexError),
    UnexpectedEndOfFile,
    UnexpectedToken(LexToken),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseContext {
    Basefile,
    Node(bool)
}

type ContextStack = Vec<ParseContext>;
pub type ParseResult = Result<(ParseEvent, Position), (ParseError, Position)>;

pub struct Parser<R: Read> {
    context: ContextStack,
    ended: bool,
    lexer: Lexer<R>,
}

impl<R: Read> Parser<R> {
    pub fn parse(lexer: Lexer<R>) -> Self {
        Parser {
            context: ContextStack::new(),
            ended: false,
            lexer: lexer,
        }
    }

    fn lex_error(&mut self, error: LexError) -> Option<ParseResult> {
        self.yield_error(ParseError::LexError(error))
    }

    fn parse_context_file(&mut self) -> Option<ParseResult> {
        let next = self.lexer.next();
        if let Some(Ok(LexToken::Identifier(ident))) = next {
            let next = self.lexer.next();
            match next {
                Some(Ok(LexToken::OpenBrace)) => {
                    self.context.push(ParseContext::Node(true));
                    self.yield_state(ParseEvent::NodeStart(ident))
                }
                Some(Ok(tok)) =>
                    self.yield_error(ParseError::UnexpectedToken(tok)),
                Some(Err(err)) =>
                    self.yield_error(ParseError::LexError(err)),
                None =>
                    self.yield_error(ParseError::UnexpectedEndOfFile),
            }
        } else if let Some(Ok(tok)) = next {
            self.yield_error(ParseError::UnexpectedToken(tok))
        } else if let Some(Err(next)) = next {
            self.lex_error(next)
        } else {
            self.ended = true;
            self.yield_state(ParseEvent::EndOfFile)
        }
    }

    fn parse_context_node(&mut self) -> Option<ParseResult> {
        let next = self.lexer.next();
        match next {
            Some(Ok(LexToken::CloseBrace)) => {
                self.context.pop();
                self.yield_state(ParseEvent::NodeEnd)
            },
            Some(Ok(tok)) => {
                self.yield_error(ParseError::NotYetImplemented)
            },
            Some(Err(err)) => {
                self.yield_error(ParseError::LexError(err))
            },
            None => {
                self.yield_error(ParseError::UnexpectedEndOfFile)
            }
        }
    }

    fn parse_context_node_no_comma(&mut self) -> Option<ParseResult> {
        let next = self.lexer.next();
        None
    }

    fn yield_state(&mut self, state: ParseEvent) -> Option<ParseResult> {
        Some(Ok((state, self.lexer.position.clone())))
    }

    fn yield_error(&mut self, error: ParseError) -> Option<ParseResult> {
        self.ended = true;
        Some(Err((error, self.lexer.position.clone())))
    }
}

impl<R: Read> Iterator for Parser<R> {
    type Item = ParseResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended { return None; }

        let current_state = self.context.pop();
        println!("state: {:?}", current_state);
        match current_state {
            None => {
                self.context.push(ParseContext::Basefile);
                self.yield_state(ParseEvent::BeginFile)
            }
            Some(ParseContext::Basefile) => {
                self.context.push(current_state.unwrap());
                self.parse_context_file()
            },
            Some(ParseContext::Node(true)) => {
                self.context.push(current_state.unwrap());
                self.parse_context_node()
            },
            Some(ParseContext::Node(false)) => {
                self.context.push(current_state.unwrap());
                self.parse_context_node_no_comma()
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
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());
    }

    #[test]
    fn handle_node() {
        let file = Cursor::new("node { }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        println!("BeginFile");
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        println!("NodeStart");
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        println!("NodeEnd");
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        println!("EndOfFile");
        assert!(parser.next().is_none());
    }
}
