use std::io::Cursor;
use std::io::prelude::*;
use std::iter::Peekable;

use super::parser::{Parser, ParseEvent, ParseError};
use super::lexer::Lexer;
use super::position::Position;

use super::types::*;

pub struct Figtree {
    parser: Peekable<Parser>,
}

impl Figtree {
    pub fn new<T: Read + 'static>(input: T) -> Self {
        Figtree {
            parser: Parser::parse(Lexer::lex(input)).peekable()
        }
    }

    pub fn from_string<T>(input: T) -> Figtree where T: Into<String> {
        Figtree::new(Cursor::new(input.into().into_bytes()))
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
                    unreachable!("ParseEvent {:?} occurred that cannot happen at this time.", ev),
                Some(Err(error)) => { return Some(error) },
                None =>
                    unreachable!("EOF occurred that cannot happen at this time."),
            }
        }
    }

    fn parse_node(&mut self, node: &mut Node) -> Option<(ParseError, Position)> {
        loop {
            match self.parser.next() {
                Some(Ok((ParseEvent::NodeEnd, _))) => { return None; },
                Some(Ok((ParseEvent::NodeStart(name), _))) => {
                    if let Some(err) = self.parse_node(node.new_node(name)) {
                        return Some(err);
                    }
                },
                Some(Ok((ParseEvent::Key(key), _))) => {
                    match self.parse_value() {
                        Ok(value) => { node.attributes.insert(key, value); },
                        Err(err) => { return Some(err); }
                    }
                }
                Some(Ok(ev)) =>
                    unreachable!("ParseEvent {:?} occurred that cannot happen at this time.", ev),
                Some(Err(error)) => { return Some(error) },
                None =>
                    unreachable!("EOF occurred that cannot happen at this time."),
            }
        }
    }

    fn parse_value(&mut self) -> Result<Value, (ParseError, Position)> {
        match self.parser.next() {
            Some(Ok((ParseEvent::Value(val), _))) =>
                Ok(Value::from_parsed_value(val)),
            Some(Ok((ParseEvent::ListStart, _))) =>
                self.parse_list(),
            Some(Ok((ParseEvent::DictStart, _))) =>
                self.parse_dict(),
            Some(Ok(ev)) =>
                unreachable!("ParseEvent {:?} occurred that cannot happen at this time.", ev),
            Some(Err(error)) =>
                Err(error),
            None =>
                unreachable!("EOF occurred that cannot happen at this time."),
        }
    }

    fn parse_list(&mut self) -> Result<Value, (ParseError, Position)> {
        let mut list = List::new();
        loop {
            if matches!(self.parser.peek(), Some(&Ok((ParseEvent::ListEnd, _)))) {
                self.parser.next();
                return Ok(Value::List(list));
            } else {
                match self.parse_value() {
                    Ok(val) => list.push(val),
                    Err(err) => { return Err(err); }
                }
            }
        }
    }

    fn parse_dict(&mut self) -> Result<Value, (ParseError, Position)> {
        let mut dict = Dict::new();
        loop {
            match self.parser.next() {
                Some(Ok((ParseEvent::Key(key), _))) => {
                    match self.parse_value() {
                        Ok(value) => { dict.insert(key, value); },
                        Err(err) => { return Err(err); }
                    }
                },
                Some(Ok((ParseEvent::DictEnd, _))) => {
                    return Ok(Value::Dict(dict));
                },
                Some(Ok(ev)) =>
                    unreachable!("ParseEvent {:?} occurred that cannot happen at this time.", ev),
                Some(Err(error)) => { return Err(error) },
                None =>
                    unreachable!("EOF occurred that cannot happen at this time."),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Figtree;
    use super::super::types::*;
    use std::collections::HashMap;

    #[test]
    fn construct_empty_file() {
        let mut figgy = Figtree::from_string("");
        let config = figgy.parse().unwrap();
        assert_eq!(config.nodes.len(), 0);
    }

    #[test]
    fn construct_empty_nodes_in_file() {
        let mut figgy = Figtree::from_string("node {} hello { nested {} }");
        let config = figgy.parse().unwrap();
        assert_eq!(config.nodes.len(), 2);
    }

    #[test]
    fn construct_key_value_pairs_from_file() {
        let mut figgy = Figtree::from_string(
            "node { 'str': 's', 'int': 5, 'float': 3.4, 'bool': true, 'ident': !gh }");
        let config = figgy.parse().unwrap();
        let node = config.get_node("node").unwrap();
        assert_eq!(node.get_attr("str").unwrap(), &Value::new_string("s"));
        assert_eq!(node.get_attr("int").unwrap(), &Value::new_int(5));
        assert_eq!(node.get_attr("float").unwrap(), &Value::new_float(3.4));
        assert_eq!(node.get_attr("bool").unwrap(), &Value::new_bool(true));
        assert_eq!(node.get_attr("ident").unwrap(), &Value::new_ident("gh"));

        let mut figgy = Figtree::from_string(
            "node {
                'list': [1, 'two', 3.0, !four, true, []],
                'dict': {
                    'str': 's', 'int': 5, 'float': 3.4, 'bool': true,
                    'ident': !gh, 'list': [], 'dict': {}
                }
            }");
        let config = figgy.parse().unwrap();
        let node = config.get_node("node").unwrap();
        assert_eq!(node.get_attr("list").unwrap(), &Value::List(vec![
            Value::new_int(1), Value::new_string("two"), Value::new_float(3.0),
            Value::new_ident("four"), Value::new_bool(true), Value::List(Vec::new())]));
        assert_eq!(node.get_attr("dict").unwrap(), &Value::Dict({
            let mut dict = Dict::new();
            dict.insert("str".to_string(), Value::new_string("s"));
            dict.insert("int".to_string(), Value::new_int(5));
            dict.insert("float".to_string(), Value::new_float(3.4));
            dict.insert("bool".to_string(), Value::new_bool(true));
            dict.insert("ident".to_string(), Value::new_ident("gh"));
            dict.insert("list".to_string(), Value::List(Vec::new()));
            dict.insert("dict".to_string(), Value::Dict(HashMap::new()));
            dict
        }));
    }
}
