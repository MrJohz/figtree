use super::lexer::{Lexer, LexToken, LexError};
use super::position::Position;

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Ident(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseEvent {
    FileStart,
    FileEnd,
    NodeStart(String),
    NodeEnd,
    Key(String),
    Value(ParsedValue),
    ListStart,
    ListEnd,
    DictStart,
    DictEnd,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    LexError(LexError),
    UnexpectedEndOfFile,
    UnexpectedToken(LexToken),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseContext {
    Basefile,
    Node(bool),
    Value,
    List(bool),
    Dict(bool),
}

type ContextStack = Vec<ParseContext>;
pub type ParseResult = Result<(ParseEvent, Position), (ParseError, Position)>;

pub struct Parser {
    context: ContextStack,
    ended: bool,
    lexer: Lexer,
}

impl Parser {
    pub fn parse(lexer: Lexer) -> Self {
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
            Some(&ParseContext::List(has_comma)) => has_comma,
            Some(&ParseContext::Dict(has_comma)) => has_comma,
        }
    }

    fn set_comma(&mut self, state: bool) {
        let pushable = match self.context.pop() {
            None => None,
            Some(ParseContext::Node(_)) => Some(ParseContext::Node(state)),
            Some(ParseContext::List(_)) => Some(ParseContext::List(state)),
            Some(ParseContext::Dict(_)) => Some(ParseContext::Dict(state)),
            Some(ctx) => Some(ctx),
        };

        if pushable.is_some() {
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
            self.yield_state(ParseEvent::FileEnd)
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
                        self.yield_state(ParseEvent::Key(key))
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
        self.context.pop();
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
                self.yield_state(ParseEvent::Value(ParsedValue::Str(val_string)))
            },
            Some(Ok(LexToken::IntegerLit(integer))) => {
                self.yield_state(ParseEvent::Value(ParsedValue::Int(integer)))
            }
            Some(Ok(LexToken::FloatLit(flt))) => {
                self.yield_state(ParseEvent::Value(ParsedValue::Float(flt)))
            }
            Some(Ok(LexToken::Identifier(ident))) => {
                match &*ident {
                    "true" => {
                        self.yield_state(ParseEvent::Value(ParsedValue::Bool(true)))
                    },
                    "false" => {
                        self.yield_state(ParseEvent::Value(ParsedValue::Bool(false)))
                    },
                    _ => {
                        self.yield_error(ParseError::UnexpectedToken(LexToken::Identifier(ident)))
                    }
                }
            },
            Some(Ok(LexToken::Bang)) => {
                match self.lexer.next() {
                    Some(Ok(LexToken::Identifier(s))) => {
                        self.yield_state(ParseEvent::Value(ParsedValue::Ident(s)))
                    },
                    Some(Ok(tok)) => self.yield_error(ParseError::UnexpectedToken(tok)),
                    Some(Err(err)) => self.lex_error(err),
                    None => self.yield_error(ParseError::UnexpectedEndOfFile),
                }
            },
            Some(Ok(LexToken::OpenBracket)) => {
                self.context.push(ParseContext::List(true));
                self.yield_state(ParseEvent::ListStart)
            },
            Some(Ok(LexToken::OpenBrace)) => {
                self.context.push(ParseContext::Dict(true));
                self.yield_state(ParseEvent::DictStart)
            },
            Some(Ok(tok)) => self.yield_error(ParseError::UnexpectedToken(tok)),
        };

        if matches!(self.lexer.peek(), Some(&Ok(LexToken::Comma))) {
            self.set_comma(true);
            self.lexer.next();
        }

        response
    }

    fn parse_context_list(&mut self) -> Option<ParseResult> {
        if matches!(self.lexer.peek(), Some(&Ok(LexToken::CloseBracket))) {
            self.lexer.next(); // consume close-bracket
            self.context.pop();
            if matches!(self.lexer.peek(), Some(&Ok(LexToken::Comma))) {
                self.set_comma(true);
                self.lexer.next();
            }
            self.yield_state(ParseEvent::ListEnd)
        } else {
            // This isn't a close-bracket, so push a value context
            // and parse the next token(s) as a value.
            self.context.push(ParseContext::Value);
            self.parse_context_value()
        }
    }

    fn parse_context_dict(&mut self) -> Option<ParseResult> {
        match self.lexer.next() {
            Some(Ok(LexToken::CloseBrace)) => {
                self.context.pop();
                self.yield_state(ParseEvent::DictEnd)
            },
            Some(Ok(LexToken::StringLit(key))) => {
                if !self.has_comma() {
                    return self.yield_error(ParseError::UnexpectedToken(LexToken::StringLit(key)));
                }
                self.set_comma(false);
                match self.lexer.next() {
                    Some(Ok(LexToken::Colon)) => {
                        self.context.push(ParseContext::Value);
                        self.yield_state(ParseEvent::Key(key))
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

    fn yield_state(&mut self, state: ParseEvent) -> Option<ParseResult> {
        Some(Ok((state, self.lexer.token_start.clone())))
    }

    fn yield_error(&mut self, error: ParseError) -> Option<ParseResult> {
        self.ended = true;
        Some(Err((error, self.lexer.token_start.clone())))
    }
}

impl Iterator for Parser {
    type Item = ParseResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended { return None; }

        let current_state = self.context.pop();
        match current_state {
            None => {
                self.context.push(ParseContext::Basefile);
                self.yield_state(ParseEvent::FileStart)
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
            },
            Some(ParseContext::List(_)) => {
                self.context.push(current_state.unwrap());
                self.parse_context_list()
            },
            Some(ParseContext::Dict(_)) => {
                self.context.push(current_state.unwrap());
                self.parse_context_dict()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lexer::{Lexer, LexToken};
    use std::io::Cursor;

    #[test]
    fn handle_empty_file() {
        let file = Cursor::new("".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());
    }

    #[test]
    fn handle_node() {
        let file = Cursor::new("node { }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());
    }

    #[test]
    fn handle_nested_node() {
        let file = Cursor::new("node { subnode {} }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("subnode".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);

        // arbitrary depth of stack
        let file = Cursor::new("node { subnode { sub { sub { sub {} } } } }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
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
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
    }

    #[test]
    fn handle_key_value_pair() {
        let file = Cursor::new("node { 'key': 'value' }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Str("value".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': 3 }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Int(3)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': 3.5 }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Float(3.5)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': true }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Bool(true)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': false }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Bool(false)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { 'key': !my_ident }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Ident("my_ident".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());
    }

    #[test]
    fn concats_string_values() {
        let file = Cursor::new("node { 'key': 'value 1' 'value 2' }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Str("value 1value 2".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());
    }

    #[test]
    fn requires_comma_between_key_value_pairs() {
        // with
        let file = Cursor::new("node { 'key1': true, 'key2': 'val' }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key1".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Bool(true)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key2".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Str("val".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());

        // without
        let file = Cursor::new("node { 'key1': true 'key2': 'val' }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key1".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Bool(true)));
        assert_eq!(parser.next().unwrap().unwrap_err().0, ParseError::UnexpectedToken(LexToken::StringLit("key2".to_string())));
        assert!(parser.next().is_none());
        let file = Cursor::new("node { 'key1': 'true' 'key2': 'val' }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key1".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Str("truekey2".to_string())));
        assert_eq!(parser.next().unwrap().unwrap_err().0, ParseError::UnexpectedToken(LexToken::Colon));
        assert!(parser.next().is_none());
    }

    #[test]
    fn handle_list_values() {
        let file = Cursor::new("node { 'key': ['val1', 2, 3.4, false, !ident] }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Str("val1".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Int(2)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Float(3.4)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Bool(false)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Ident("ident".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());
    }

    #[test]
    fn handle_nested_lists() {
        let file = Cursor::new("node { 'key': ['lista', ['listb', []]] }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Str("lista".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Str("listb".to_string())));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());
    }

    #[test]
    fn trailing_commas() {
        let file = Cursor::new("node { 'key': [1, 2,], subnode {} }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Int(1)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Int(2)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::ListEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("subnode".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());

        let file = Cursor::new("node { , }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap_err().0, ParseError::UnexpectedToken(LexToken::Comma));
        assert!(parser.next().is_none());
    }

    #[test]
    fn handles_dict_values() {
        let file = Cursor::new("node { 'key': {'1': 2, '3': 4} }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::DictStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("1".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Int(2)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("3".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Value(ParsedValue::Int(4)));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::DictEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());
    }

    #[test]
    fn handle_nested_dicts() {
        let file = Cursor::new("node { 'key': {'1': {'b': {} } } }".as_bytes());
        let mut parser = Parser::parse(Lexer::lex(file));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeStart("node".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("key".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::DictStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("1".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::DictStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::Key("b".to_string()));
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::DictStart);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::DictEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::DictEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::DictEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::NodeEnd);
        assert_eq!(parser.next().unwrap().unwrap().0, ParseEvent::FileEnd);
        assert!(parser.next().is_none());
    }
}
