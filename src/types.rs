//! A collection of types that define a figtree document.
//!
//! These types are re-exported in the main module because there aren't too many of
//! them, and because they're useful when testing equality or building figtree documents
//! from scratch.  The `types` module is also made available to allow explicit namespaced
//! imports of these types.
//!
//! # Examples
//! ```
//! # use figtree::types::*;
//!
//! let mut doc = Document::new();
//! let mut node = doc.new_node("node_name");
//! node.attributes.insert(
//!     "key".to_string(),
//!     Value::new_int(4032));
//! ```

use std::collections::HashMap;
use super::parser::ParsedValue;

/// A type to represent a figtree dict
///
/// Maps string keys to `Value`s.  Can contain any `Value`, including container types
pub type Dict = HashMap<String, Value>;

/// A type to represent a figtree list
///
/// Contains ordered `Value`s.  Can contain any `Value`, including container types
pub type List = Vec<Value>;

/// A type to represent a figtree value
///
/// Generally, this is obtained by getting the value of an attribute on a Node, although
/// there are also methods to construct all the kinds of this type.
///
/// # Examples
/// ```
/// # use figtree::types::Value;
/// let value = Value::new_string("hello!");
/// assert!(value.get_str() == Some("hello!"));
/// assert!(value.get_int() == None);
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
    /// Construct a new string `Value`.
    pub fn new_string<S>(s: S) -> Self where S: Into<String> {
        Value::Str(s.into())
    }

    /// Construct a new identifier `Value`.
    pub fn new_ident<S>(s: S) -> Self where S: Into<String> {
        Value::Ident(s.into())
    }

    /// Construct a new integer `Value`.
    pub fn new_int(s: i64) -> Self {
        Value::Int(s)
    }

    /// Construct a new float `Value`.
    pub fn new_float(s: f64) -> Self {
        Value::Float(s)
    }

    /// Construct a new boolean `Value`.
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

    /// Extract the contained value if it is a string.
    pub fn get_str(&self) -> Option<&str> {
        match *self {
            Value::Str(ref s) => Some(&s),
            _ => None
        }
    }

    /// Extract the contained value if it is an integer
    pub fn get_int(&self) -> Option<i64> {
        match *self {
            Value::Int(s) => Some(s),
            _ => None
        }
    }

    /// Extract the contained value if it is a float
    pub fn get_float(&self) -> Option<f64> {
        match *self {
            Value::Float(s) => Some(s),
            _ => None
        }
    }

    /// Extract the contained value if it is a boolean
    pub fn get_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(s) => Some(s),
            _ => None
        }
    }

    /// Extract the contained (&str) value if it is an identifier
    pub fn get_ident(&self) -> Option<&str> {
        match *self {
            Value::Ident(ref s) => Some(&s),
            _ => None
        }
    }

    /// Extract the contained value if it is a dict
    pub fn get_dict(&self) -> Option<&Dict> {
        match *self {
            Value::Dict(ref s) => Some(s),
            _ => None
        }
    }

    /// Extract the contained value as a slice if it is a list
    pub fn get_list(&self) -> Option<&[Value]> {
        match *self {
            Value::List(ref s) => Some(&s),
            _ => None
        }
    }
}

/// A struct representing an individual node in a parsed document
///
/// # Examples
///
/// ```
/// use figtree::types::*;
/// let mut node = Node::new();
/// assert!(node.subnodes.len() == 0);  // empty
/// assert!(node.attributes.len() == 0);
///
/// node.new_node("subnode");
///
/// { // appease the borrow checker
///     let subnode = node.get_node("subnode").expect("no such node");
///     assert!(subnode == &Node::new());  // creates a blank subnode
/// }
///
/// { // appease the borrow checker
///     let mut subnode = node.get_node_mut("subnode").expect("no such node");
///     let sub_subnode = subnode.new_node("sub_subnode");
///     // etc.
/// }
/// ```
#[derive(Debug, PartialEq)]
pub struct Node {
    pub subnodes: HashMap<String, Node>,
    pub attributes: HashMap<String, Value>,
}

impl Node {
    /// Construct a new, empty node
    pub fn new() -> Self {
        Node {
            subnodes: HashMap::new(),
            attributes: HashMap::new(),
        }
    }

    /// Insert a new subnode into this node, and return a mutable reference to it
    pub fn new_node<S>(&mut self, name: S) -> &mut Self
        where S: Into<String> + Clone {

        let key = name.clone().into();
        self.subnodes.insert(name.into(), Self::new());
        self.subnodes.get_mut(&key).unwrap()
    }

    /// Get a reference to the specified subnode
    pub fn get_node<S>(&self, name: S) -> Option<&Self> where S: Into<String> {
        self.subnodes.get(&name.into())
    }

    /// Get a mutable reference to the specified subnode
    pub fn get_node_mut<S>(&mut self, name: S) -> Option<&mut Self>
        where S: Into<String> {

        self.subnodes.get_mut(&name.into())
    }

    /// Get a reference to the specified attribute value
    pub fn get_attr<S>(&self, name: S) -> Option<&Value> where S: Into<String> {
        self.attributes.get(&name.into())
    }

    /// Get a mutable reference to the specified attribute value
    pub fn get_attr_mut<S>(&mut self, name: S) -> Option<&mut Value>
        where S: Into<String> {

        self.attributes.get_mut(&name.into())
    }
}

/// A struct representing a parsed figtree document.
///
/// # Examples
///
/// ```
/// use figtree::types::*;
/// let mut doc = Document::new();
/// doc.new_node("node name");
/// assert!(doc.nodes.len() == 1);
///
/// {   // appease the borrow checker by scoping this off
///     // in real code this would be unnecessary because there is no need to do
///     // something so pathological.
///     let node = doc.get_node("node name").expect("no such node");
///     assert!(node == &Node::new()); // new node is a fresh, blank node
/// }
///
/// {   // again, appease the borrow checker
///     let mut node = doc.get_node_mut("node name").expect("no such node");
///     // node can be modified here
/// }
/// ```
#[derive(Debug, PartialEq)]
pub struct Document {
    pub nodes: HashMap<String, Node>,
}

impl Document {
    /// Construct a new, empty document
    pub fn new() -> Self {
        Document {
            nodes: HashMap::new(),
        }
    }

    /// Insert a node into the document.
    ///
    /// It must be possible to clone the node name, and turn it into a String.
    /// This returns a mutable reference to the Node, because I assume in most cases
    /// the desire would be to immediately start modifying the node that has just been
    /// created.
    pub fn new_node<S>(&mut self, name: S) -> &mut Node
        where S: Into<String> + Clone {

        let key = name.clone().into();
        self.nodes.insert(name.into(), Node::new());
        self.nodes.get_mut(&key).unwrap()
    }

    /// Get a reference to a specified node
    ///
    /// Essentially a thin wrapper around the `Document.nodes` mapping, but it allows for
    /// &str arguments, and allows users to do common operations without having to know
    /// about the internal structure of the node.
    pub fn get_node<S>(&self, name: S) -> Option<&Node> where S: Into<String> {
        self.nodes.get(&name.into())
    }

    /// Get a mutable reference to a specified node
    ///
    /// Essentially a thin wrapper around the `Document.nodes` mapping, but it allows for
    /// &str arguments, and allows users to do common operations without having to know
    /// about the internal structure of the node.
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
