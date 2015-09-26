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
//! let mut node = doc.new_node_or_get("node_name");
//! node.insert_attr(
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
    Null,
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

    /// Construct a new null `Value`.
    pub fn new_null() -> Self {
        Value::Null
    }

    pub fn from_parsed_value(val: ParsedValue) -> Self {
        match val {
            ParsedValue::Str(s) => Self::new_string(s),
            ParsedValue::Float(f) => Self::new_float(f),
            ParsedValue::Bool(b) => Self::new_bool(b),
            ParsedValue::Int(i) => Self::new_int(i),
            ParsedValue::Ident(i) => Self::new_ident(i),
            ParsedValue::Null => Self::new_null(),
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

    pub fn is_null(&self) -> bool {
        match *self {
            Value::Null => true,
            _ => false,
        }
    }
}

/// A struct representing an individual node in a parsed document
///
/// # Examples
///
/// Manipulating subnodes:
///
/// ```
/// use figtree::types::*;
/// let mut node = Node::new();
/// assert!(node.is_empty());
/// assert!(!node.has_nodes());
///
/// node.new_node_or_get("subnode");
/// assert!(node.node_count() == 1);
///
/// { // appease the borrow checker
///     let subnode = node.get_node("subnode").expect("no such node");
///     assert!(subnode == &Node::new());  // creates a blank subnode
/// }
///
/// { // appease the borrow checker
///     let mut subnode = node.get_node_mut("subnode").expect("no such node");
///     let sub_subnode = subnode.new_node_or_get("sub_subnode");
///     // etc.
/// }
/// ```
///
/// Manipulating attributes
///
/// ```
/// use figtree::types::*;
/// let mut node = Node::new();
/// assert!(node.is_empty());
/// assert!(!node.has_attrs());
///
/// node.insert_attr("key", Value::new_int(5));
/// assert!(node.attr_count() == 1);
/// ```
#[derive(Debug, PartialEq)]
pub struct Node {
    subnodes: HashMap<String, Node>,
    attributes: HashMap<String, Value>,
}

impl Node {
    /// Construct a new, empty node
    pub fn new() -> Self {
        Node {
            subnodes: HashMap::new(),
            attributes: HashMap::new(),
        }
    }

    /// Construct a new node and automatically insert it as a subnode.
    ///
    /// Returns a mutable reference to the new node.  If there is a subnode already
    /// present with the given name, this method will not insert a new node and instead
    /// just return the old node.
    pub fn new_node_or_get<S>(&mut self, name: S) -> &mut Self where S: Into<String> {
        self.subnodes.entry(name.into()).or_insert(Self::new())
    }

    /// Inserts a node into this node as a subnode.
    ///
    /// If there is already a node with the given name, replace it and return the
    /// old node.
    pub fn insert_node<S>(&mut self, name: S, node: Node) -> Option<Node>
        where S: Into<String> {

        self.subnodes.insert(name.into(), node)
    }

    /// Remove a subnode from this node.
    ///
    /// Returns the deleted node.
    pub fn delete_node<S>(&mut self, name: S) -> Option<Node> where S: Into<String> {
        self.subnodes.remove(&name.into())
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

    /// Insert a new value into this node.
    ///
    /// If there is already a value with the given name, replace it and return the old
    /// value.
    pub fn insert_attr<S>(&mut self, name: S, value: Value) -> Option<Value>
        where S: Into<String> {

        self.attributes.insert(name.into(), value)
    }


    /// Remove an attribute from this node.
    ///
    /// Returns the deleted value.
    pub fn delete_attr<S>(&mut self, name: S) -> Option<Value> where S: Into<String> {
        self.attributes.remove(&name.into())
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

    /// Test if this node has no subnodes or attributes
    pub fn is_empty(&self) -> bool {
        self.subnodes.is_empty() && self.attributes.is_empty()
    }

    /// Test if this node has a subnode with the given name.
    pub fn has_node(&self, name: &String) -> bool {
        self.subnodes.contains_key(name)
    }

    /// Test if this node has any subnodes at all.
    pub fn has_nodes(&self) -> bool {
        !self.subnodes.is_empty()
    }

    /// Returns the number of subnodes.
    pub fn node_count(&self) -> usize {
        self.subnodes.len()
    }

    /// Test if this node had an attribute with the given key.
    pub fn has_attr(&self, name: &String) -> bool {
        self.attributes.contains_key(name)
    }

    /// Test if this node has any attributes at all.
    pub fn has_attrs(&self) -> bool {
        !self.attributes.is_empty()
    }

    /// Returns the number of attributes.
    pub fn attr_count(&self) -> usize {
        self.attributes.len()
    }
}

/// A struct representing a parsed figtree document.
///
/// # Examples
///
/// ```
/// use figtree::types::*;
/// let mut doc = Document::new();
/// assert!(doc.is_empty());
///
/// doc.new_node_or_get("node name");
/// assert!(doc.node_count() == 1);
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
    nodes: HashMap<String, Node>,
}

impl Document {
    /// Construct a new, empty document
    pub fn new() -> Self {
        Document {
            nodes: HashMap::new(),
        }
    }

    /// Construct a new node and insert it into the document.
    ///
    /// Returns a mutable reference to the new node.  If there is a node already
    /// present with the given name, this method will not insert a new node and instead
    /// just return the old node.
    pub fn new_node_or_get<S>(&mut self, name: S) -> &mut Node where S: Into<String> {
        self.nodes.entry(name.into()).or_insert(Node::new())
    }

    /// Inserts a node into the document.
    ///
    /// If there is already a node with the given name, replace it and return the
    /// old node.
    pub fn insert_node<S>(&mut self, name: S, node: Node) -> Option<Node>
        where S: Into<String> {

        self.nodes.insert(name.into(), node)
    }

    /// Remove a node from the document.
    ///
    /// Returns the deleted node, if it exists.
    pub fn delete_node<S>(&mut self, name: S) -> Option<Node> where S: Into<String> {
        self.nodes.remove(&name.into())
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

    /// Test if the document is empty - if it has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Test if the document has a given node.
    pub fn has_node(&self, name: &String) -> bool {
        self.nodes.contains_key(name)
    }

    /// Test if the document has any nodes.
    pub fn has_nodes(&self) -> bool {
        !self.nodes.is_empty()
    }

    /// Returns the number of nodes in the document.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
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
    fn node_with_subnodes() {
        let mut node = Node::new();
        assert!(node.is_empty());
        assert!(!node.has_nodes());
        assert!(!node.has_attrs());
        assert_eq!(node.node_count(), 0);
        assert_eq!(node.attr_count(), 0);

        node.new_node_or_get("subnode_name").new_node_or_get("secondary_subnode");
        assert_eq!(
            node.get_node("subnode_name")
                .expect("couldn't find subnode_name")
                .get_node("secondary_subnode"),
            Some(&Node::new()));
        assert!(!node.is_empty());
        assert!(node.has_nodes());
        assert!(!node.has_attrs());
        assert_eq!(node.node_count(), 1);
        assert_eq!(node.attr_count(), 0);

        let subnode = node.delete_node("subnode_name").expect("node should have existed");
        assert!(node.is_empty());
        assert!(!node.has_nodes());
        assert!(!node.has_attrs());
        assert_eq!(node.node_count(), 0);
        assert_eq!(node.attr_count(), 0);

        node.insert_node("new subnode", subnode);
        assert!(!node.is_empty());
        assert!(node.has_nodes());
        assert!(!node.has_attrs());
        assert_eq!(node.node_count(), 1);
        assert_eq!(node.attr_count(), 0);
    }

    #[test]
    fn node_with_attributes() {
        let mut node = Node::new();
        assert!(node.is_empty());
        assert!(!node.has_nodes());
        assert!(!node.has_attrs());
        assert_eq!(node.node_count(), 0);
        assert_eq!(node.attr_count(), 0);

        assert_eq!(node.insert_attr("key", Value::new_int(6)), None);
        assert!(!node.is_empty());
        assert!(!node.has_nodes());
        assert!(node.has_attrs());
        assert_eq!(node.node_count(), 0);
        assert_eq!(node.attr_count(), 1);

        assert_eq!(node.delete_attr("key"), Some(Value::new_int(6)));
        assert!(node.is_empty());
        assert!(!node.has_nodes());
        assert!(!node.has_attrs());
        assert_eq!(node.node_count(), 0);
        assert_eq!(node.attr_count(), 0);

    }

    #[test]
    fn document_tests() {
        let mut doc = Document::new();
        doc.new_node_or_get("node_name").new_node_or_get("subnode");
        assert_eq!(
            doc.get_node("node_name")
                .expect("couldn't find node_name")
                .get_node("subnode"),
            Some(&Node::new()));
    }
}
