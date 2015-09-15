use std::collections::HashMap;
use super::parser::ParsedValue;

pub type Dict = HashMap<String, Value>;
pub type List = Vec<Value>;

#[derive(Debug, PartialEq)]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Ident(String),
    Dict(Dict),
    List(List),
}

impl Value {
    pub fn new_string<S>(s: S) -> Self where S: Into<String> {
        Value::Str(s.into())
    }

    pub fn new_ident<S>(s: S) -> Self where S: Into<String> {
        Value::Ident(s.into())
    }

    pub fn new_int(s: i64) -> Self {
        Value::Int(s)
    }

    pub fn new_float(s: f64) -> Self {
        Value::Float(s)
    }

    pub fn new_bool(s: bool) -> Self {
        Value::Bool(s)
    }

    pub fn from_parsed_value(val: ParsedValue) -> Self {
        match val {
            ParsedValue::Str(s) => Self::new_string(s),
            ParsedValue::Float(f) => Self::new_float(f),
            ParsedValue::Bool(b) => Self::new_bool(b),
            ParsedValue::Int(i) => Self::new_int(i),
            ParsedValue::Ident(i) => Self::new_ident(i),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Node {
    pub subnodes: HashMap<String, Node>,
    pub attributes: HashMap<String, Value>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            subnodes: HashMap::new(),
            attributes: HashMap::new(),
        }
    }

    pub fn new_node<S>(&mut self, name: S) -> &mut Self
        where S: Into<String> + Clone {

        let key = name.clone().into();
        self.subnodes.insert(name.into(), Self::new());
        self.subnodes.get_mut(&key).unwrap()
    }

    pub fn get_node<S>(&self, name: S) -> Option<&Self> where S: Into<String> {
        self.subnodes.get(&name.into())
    }

    pub fn get_node_mut<S>(&mut self, name: S) -> Option<&mut Self>
        where S: Into<String> {

        self.subnodes.get_mut(&name.into())
    }

    pub fn get_attr<S>(&self, name: S) -> Option<&Value> where S: Into<String> {
        self.attributes.get(&name.into())
    }

    pub fn get_attr_mut<S>(&mut self, name: S) -> Option<&mut Value>
        where S: Into<String> {

        self.attributes.get_mut(&name.into())
    }
}

#[derive(Debug, PartialEq)]
pub struct Document {
    pub nodes: HashMap<String, Node>,
}

impl Document {
    pub fn new() -> Self {
        Document {
            nodes: HashMap::new(),
        }
    }

    pub fn new_node<S>(&mut self, name: S) -> &mut Node
        where S: Into<String> + Clone {

        let key = name.clone().into();
        self.nodes.insert(name.into(), Node::new());
        self.nodes.get_mut(&key).unwrap()
    }

    pub fn get_node<S>(&self, name: S) -> Option<&Node> where S: Into<String> {
        self.nodes.get(&name.into())
    }

    pub fn get_node_mut<S>(&mut self, name: S) -> Option<&mut Node> where S: Into<String> {
        self.nodes.get_mut(&name.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_creations() {
        let doc = Document::new();
        assert_eq!(doc.nodes.len(), 0);

        let node = Node::new();
        assert_eq!(node.subnodes.len(), 0);
        assert_eq!(node.attributes.len(), 0);

        let string = Value::new_string("hello");
        assert_eq!(string, Value::Str("hello".to_string()));
        assert_eq!(string, Value::new_string("hello".to_string()));

        let identifier = Value::new_ident("hello");
        assert_eq!(identifier, Value::Ident("hello".to_string()));
        assert_eq!(identifier, Value::new_ident("hello".to_string()));

        let integer = Value::new_int(34);
        assert_eq!(integer, Value::Int(34));

        let floatval = Value::new_float(33.4);
        assert_eq!(floatval, Value::Float(33.4));

        let boolean = Value::new_bool(false);
        assert_eq!(boolean, Value::Bool(false));
    }

    #[test]
    fn node_tests() {
        let mut node = Node::new();
        node.new_node("subnode_name").new_node("secondary_subnode");
        assert_eq!(
            node.get_node("subnode_name")
                .expect("couldn't find subnode_name")
                .get_node("secondary_subnode"),
            Some(&Node::new()));
    }

    #[test]
    fn document_tests() {
        let mut doc = Document::new();
        doc.new_node("node_name").new_node("subnode");
        assert_eq!(
            doc.get_node("node_name")
                .expect("couldn't find node_name")
                .get_node("subnode"),
            Some(&Node::new()));
    }
}
