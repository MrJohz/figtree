use std::io::{Cursor, Error};
use std::fs::File;
use std::io::prelude::*;

use super::parser::{Parser, ParseEvent, ParseError};
use super::lexer::Lexer;
use super::position::Position;

use super::types::*;

/// Opens, parses, and reads figtree files.
///
/// The `Figtree` struct is essentially a wrapper around the internal pull-parser API
/// that consumes the loaded document and transforms it into a `Document` struct.
/// In future, it will also provide its own pull-parser API to allow for streamed parsing
/// of documents and other advanced processing techniques.
///
/// # Examples
///
/// ```
/// use figtree::Figtree;
/// let mut figgy = Figtree::from_string("mydoc { 'key': 'val' }");
/// let config = figgy.parse().ok().expect("Invalid document parsed");
/// assert!(config.node_count() == 1);
/// ```
pub struct Figtree {
    parser: Parser,
}

impl Figtree {
    /// Constructs a `Figtree` instance from a generic `Read` implementor.
    ///
    /// The `Read`er *must* also have a `'static` lifetime - that is, it cannot maintain
    /// references to unowned data.  This shouldn't be much of a problem in most cases,
    /// but it may catch you out.  This is the low-level method that does the hard work
    /// of constructing a Figtree instance - you may want to use either `from_filename`
    /// or `from_string` depending on your needs.
    ///
    /// # Examples
    /// ```
    /// # use figtree::Figtree;
    /// # use std::io::Cursor;
    /// let figgy = Figtree::new(Cursor::new(String::from("my_string").into_bytes()));
    /// ```
    pub fn new<T: Read + 'static>(input: T) -> Self {
        Figtree {
            parser: Parser::parse(Lexer::lex(input))
        }
    }

    /// Constructs a `Figtree` instance from a local file.
    ///
    /// Use this over directly calling `Figtree::new` unless you need control over the
    /// file-opening process.
    ///
    /// # Failures
    /// This function will fail under the same circumstances that `File::open` will fail,
    /// producing the same error (`std::io::Error`).
    pub fn from_filename<T>(input: T) -> Result<Figtree, Error> where T: Into<String> {
        Ok(Figtree::new(try!(File::open(input.into()))))
    }

    /// Constructs a `Figtree` instance from a &str or String.
    ///
    /// Use this over directly calling `Figtree::new()` unless you need control over the
    /// file-opening process.
    ///
    /// Generally, using `from_filename` is better than loading the file into memory and
    /// parsing it as a string.  That said, there may be circumstances where it is you
    /// have a string, in which case this is better than writing the string to a file
    /// and reading from that.
    ///
    /// Both an &str or a String can be used - or indeed anything that implements the
    /// `Into<String>` trait.
    ///
    /// # Examples
    /// ```
    /// # use figtree::Figtree;
    /// // N.B. syntax doesn't matter at this point as the file isn't parsed until
    /// // the parse method is called.
    /// let mut figgy = Figtree::from_string("input");
    /// ```
    pub fn from_string<T>(input: T) -> Figtree where T: Into<String> {
        Figtree::new(Cursor::new(input.into().into_bytes()))
    }

    /// Parse the document stored in this `Figtree` instance into a `Document`.
    ///
    /// # Failures
    /// If a parsing error occurs, a `(ParseError, Position)` tuple is returned, where
    /// the `ParseError` contains the kind of error that happened, and the `Position`
    /// points to the position the lexer was in at the beginning of the last (erroring)
    /// token.
    ///
    /// # Examples
    /// Parsing successfully:
    ///
    /// ```
    /// # use figtree::Figtree;
    /// let mut figgy = Figtree::from_string("doc { 'key': 'value' }");
    /// let config = figgy.parse().ok().expect("failed to parse");
    /// assert!(config.node_count() == 1);
    /// ```
    ///
    /// Parsing unsuccessfully
    ///
    /// ```
    /// # use figtree::Figtree;
    /// # use figtree::ParseError;
    /// # use figtree::LexToken;
    /// # use figtree::Position;
    /// let mut figgy = Figtree::from_string("invalid document");
    /// let error = figgy.parse().err().expect("parsing should have failed");
    /// assert_eq!(
    ///     error.0,
    ///     ParseError::UnexpectedToken(LexToken::Identifier("document".to_string())));
    /// assert_eq!(
    ///     error.1,
    ///     Position::at(0, 8));
    /// ```
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
                    if doc.has_node(&name) {
                        return Some((ParseError::RepeatedNode(name), self.parser.lex_position()));
                    }
                    if let Some(err) = self.parse_node(doc.new_node_or_get(name)) {
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
                    if node.has_node(&name) {
                        return Some((ParseError::RepeatedNode(name), self.parser.lex_position()));
                    }
                    if let Some(err) = self.parse_node(node.new_node_or_get(name)) {
                        return Some(err);
                    }
                },
                Some(Ok((ParseEvent::Key(key), _))) => {
                    match self.parse_value() {
                        Ok(value) => { node.insert_attr(key, value); },
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
        assert!(config.is_empty());
    }

    #[test]
    fn construct_empty_nodes_in_file() {
        let mut figgy = Figtree::from_string("node {} hello { nested {} }");
        let config = figgy.parse().unwrap();
        assert_eq!(config.node_count(), 2);
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
