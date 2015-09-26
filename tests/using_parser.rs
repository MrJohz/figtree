extern crate figtree;
use figtree::Figtree;

const SAMPLE: &'static str = "tests/resources/sample.ft";

#[test]
fn opening_a_file() {
    let mut figgy = Figtree::from_filename(SAMPLE).ok().expect("file does not exist");
    let config = figgy.parse().ok().expect("parsing error occurred");

    assert_eq!(config.node_count(), 1);
}

#[test]
fn getting_different_values() {
    let mut figgy = Figtree::from_filename(SAMPLE).ok().expect("file does not exist");
    let config = figgy.parse().ok().expect("parsing error occurred");

    let test_node = config.get_node("test").expect("node is not present");
    let test_value = test_node.get_attr("string").expect("key 'string' not found");
    let string = test_value.get_str().expect("value was not string type");

    assert_eq!(string, "value");

    let integer = config.get_node("test")
        .and_then(|node| node.get_attr("list"))
        .and_then(|list| list.get_list())
        .and_then(|list| list.get(2))
        .and_then(|val| val.get_int())
        .expect("could not obtain value");

    assert_eq!(integer, 3);
}

#[test]
fn getting_subnodes() {
    let mut figgy = Figtree::from_filename(SAMPLE).ok().expect("file does not exist");
    let config = figgy.parse().ok().expect("parsing error occurred");

    let test_node = config.get_node("test").expect("node is not present");
    let subnode = test_node.get_node("subtest").expect("subnode is not present");
    assert_eq!(subnode.node_count(), 0);
}

#[test]
fn using_dictionaries() {
    let mut figgy = Figtree::from_filename(SAMPLE).ok().expect("file does not exist");
    let config = figgy.parse().ok().expect("parsing error occurred");

    let identifier = config.get_node("test")
        .and_then(|node| node.get_node("subtest"))
        .and_then(|node| node.get_attr("dict"))
        .and_then(|dict| dict.get_dict())
        .and_then(|dict| dict.get("an identifier"))
        .and_then(|val| val.get_ident())
        .expect("could not obtain value");
    assert_eq!(identifier, "jello_shots");
}

#[test]
fn using_nulls() {
    let mut figgy = Figtree::from_filename(SAMPLE).ok().expect("file does not exist");
    let config = figgy.parse().ok().expect("parsing error occurred");

    let null_node = config.get_node("test")
        .and_then(|node| node.get_node("subtest"))
        .and_then(|node| node.get_attr("nonexistent"))
        .expect("attribute is not present");

    assert_eq!(null_node.get_int(), None);
    assert!(null_node.is_null());
}
