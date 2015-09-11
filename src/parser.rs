use super::lexer::{Lexer, LexToken, LexError};
use super::position::Position;
use std::io::prelude::*;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Ident(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseEvent {
    BeginFile,
    EndOfFile,
    NodeStart(String),
    NodeEnd,
    NewPair(String),
    PairedValue(Value),
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
    Node(bool),
    Value,
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

    fn has_comma(&mut self) -> bool {
        match self.context.last() {
            None => false,
            Some(&ParseContext::Basefile) => false,
            Some(&ParseContext::Value) => false,
            Some(&ParseContext::Node(has_comma)) => has_comma,
        }
    }

    fn set_comma(&mut self, state: bool) {
        let pushable = match self.context.pop() {
            None => None,
            Some(ParseContext::Node(_)) => Some(ParseContext::Node(state)),
            Some(ctx) => Some(ctx),
        };

        if !pushable.is_none() {
            self.context.push(pushable.unwrap());
        }
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
            Some(Ok(LexToken::Identifier(ident))) => {
                self.set_comma(true);
                match self.lexer.next() {
                    Some(Ok(LexToken::OpenBrace)) => {
                        self.context.push(ParseContext::Node(true));
                        self.yield_state(ParseEvent::NodeStart(ident))
                    },
                    Some(Ok(tok)) =>
                        self.yield_error(ParseError::UnexpectedToken(tok)),
                    Some(Err(err)) =>
                        self.lex_error(err),
                    None =>
                        self.yield_error(ParseError::UnexpectedEndOfFile),
                }
            },
            Some(Ok(LexToken::StringLit(key))) => {
                if !self.has_comma() {
                    return self.yield_error(ParseError::UnexpectedToken(LexToken::StringLit(key)));
                }
                self.set_comma(false);
                match self.lexer.next() {
                    Some(Ok(LexToken::Colon)) => {
                        self.context.push(ParseContext::Value);
                        self.yield_state(ParseEvent::NewPair(key))
                    },
                    Some(Ok(tok)) =>
                        self.yield_error(ParseError::UnexpectedToken(tok)),
                    Some(Err(err)) =>
                        self.lex_error(err),
                    None =>
                        self.yield_error(ParseError::UnexpectedEndOfFile),
                }
            },
            Some(Ok(tok)) => {
                self.yield_error(ParseError::UnexpectedToken(tok))
            },
            Some(Err(err)) => {
                self.yield_error(ParseError::LexError(err))
            },
            None => {
                self.yield_error(ParseError::UnexpectedEndOfFile)
            }
        }
    }

    fn parse_context_value(&mut self) -> Option<ParseResult> {
       let response = match self.lexer.next() {
            None => self.yield_error(ParseError::UnexpectedEndOfFile),
            Some(Err(err)) => self.yield_error(ParseError::LexError(err)),
            Some(Ok(LexToken::StringLit(string))) => {
                let mut val_string = String::new();
                val_string.push_str(&string);
                loop {
                    // I think this hack is necessary
                    match self.lexer.peek() {
                        Some(&Ok(LexToken::StringLit(_))) => {},
                        _ => { break; }
                    }

                    match self.lexer.next().unwrap().unwrap() {
                        LexToken::StringLit(s) => {
                            val_string.push_str(&s);
                        },
                        _ => unreachable!(),
                    }
                }
                self.context.pop();
                self.yield_state(ParseEvent::PairedValue(Value::Str(val_string)))
            },
            Some(Ok(LexToken::IntegerLit(integer))) => {
                self.context.pop();
                self.yield_state(ParseEvent::PairedValue(Value::Int(integer)))
            }
            Some(Ok(LexToken::FloatLit(flt))) => {
                self.context.pop();
                self.yield_state(ParseEvent::PairedValue(Value::Float(flt)))
            }
            Some(Ok(LexToken::Identifier(ident))) => {
                match &*ident {
                    "true" => {
                        self.context.pop();
                        self.yield_state(ParseEvent::PairedValue(Value::Bool(true)))
                    },
                    "false" => {
                        self.context.pop();
                        self.yield_state(ParseEvent::PairedValue(Value::Bool(false)))
                    },
                    _ => {
                        self.yield_error(ParseError::UnexpectedToken(LexToken::Identifier(ident)))
                    }
                }
            },
            Some(Ok(LexToken::Bang)) => {
                match self.lexer.next() {
                    Some(Ok(LexToken::Identifier(s))) => {
                        self.context.pop();
                        self.yield_state(ParseEvent::PairedValue(Value::Ident(s)))
                    },
                    Some(Ok(tok)) => self.yield_error(ParseError::UnexpectedToken(tok)),
                    Some(Err(err)) => self.lex_error(err),
                    None => self.yield_error(ParseError::UnexpectedEndOfFile),
                }
            },
            Some(Ok(tok)) => self.yield_error(ParseError::UnexpectedToken(tok)),
        };

        // weird way of doing things, but has_comma() is *false* at the moment.  If
        // the next token (unconsumed) is a comma, set it to true.  Can't consume that
        // next token while borrowing self.lexer, so in the next statement, if
        // has_comma() is *true* (that is, previous if-statement matched), consume it.
        if let Some(&Ok(LexToken::Comma)) = self.lexer.peek() {
            self.set_comma(true);
        }
        if self.has_comma() {
            self.lexer.next();
        }

        response
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
        match current_state {
            None => {
                self.context.push(ParseContext::Basefile);
                self.yield_state(ParseEvent::BeginFile)
            }
            Some(ParseContext::Basefile) => {
                self.context.push(current_state.unwrap());
                self.parse_context_file()
            },
            Some(ParseContext::Node(_)) => {
                self.context.push(current_state.unwrap());
                self.parse_context_node()
            },
            Some(ParseContext::Value) => {
                self.context.push(current_state.unwrap());
                self.parse_context_value()
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
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());
    }

    #[test]
    fn handle_nested_node() {
        let file = Cursor::new("node { subnode {} }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("subnode".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);

        // arbitrary depth of stack
        let file = Cursor::new("node { subnode { sub { sub { sub {} } } } }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("subnode".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("sub".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("sub".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("sub".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
    }

    #[test]
    fn handle_key_value_pair() {
        let file = Cursor::new("node { 'key': 'value' }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Str("value".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': 3 }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Int(3)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': 3.5 }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Float(3.5)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': true }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Bool(true)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': false }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Bool(false)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': !my_ident }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Ident("my_ident".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());
    }

    #[test]
    fn requires_comma_between_key_value_pairs() {
        // with
        let file = Cursor::new("node { 'key1': true, 'key2': 'val' }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key1".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Bool(true)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key2".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Str("val".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());

        // without
        let file = Cursor::new("node { 'key1': true 'key2': 'val' }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key1".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Bool(true)));
        assert!(parser.next().unwrap().is_err());
        assert!(parser.next().is_none());
    }

    #[test]
    fn concats_string_values() {
        let file = Cursor::new("node { 'key': 'value 1' 'value 2' }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::BeginFile);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NewPair("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::PairedValue(Value::Str("value 1value 2".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::EndOfFile);
        assert!(parser.next().is_none());
    }
}
