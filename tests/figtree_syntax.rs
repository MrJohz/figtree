extern crate figtree;
use figtree::Figtree;

const REPEATED_NODES_IN_DOCUMENT: &'static str = "
    node { 'key': 1 }
    node { 'key': 2 }";
const REPEATED_NODES_IN_NODE: &'static str = "
    node {
        subnode { 'key': 1 }
        subnode { 'key': 2 }
    }";

#[test]
fn repeated_nodes() {
    let mut figgy = Figtree::from_string(REPEATED_NODES_IN_DOCUMENT);
    let error = figgy.parse().err().expect("Parsing should have failed (document)");
    assert_eq!(error.0, figtree::ParseError::RepeatedNode("node".to_string()));

    let mut figgy = Figtree::from_string(REPEATED_NODES_IN_NODE);
    let error = figgy.parse().err().expect("Parsing should have failed (node)");
    assert_eq!(error.0, figtree::ParseError::RepeatedNode("subnode".to_string()));
}
