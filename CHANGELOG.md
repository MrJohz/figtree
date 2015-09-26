# 0.2.0 (2015-09-26)

### Breaking changes
- Privatised `attributes`, `subnodes`, and `nodes` attrs on `types::Document` and `types::Node`.  Direct access to these hashmaps have been replaced by a new set of methods that should allow for anything that was previously being done to still be done.
- Renamed `types::Document::new_node` to `types::Document::new_node_or_get`, and change its semantics slightly.  If a node already exists with the specified name, that node will be returned instead.  To always insert a new node, use `types::Document::insert_node`.  To delete a currently existing node, use `types::Document::remove_node`.
- Likewise with `types::Node::new_node` -> `types::Node::new_node_or_get`.

### API additions
- `types::Node`
    + `insert_node`
    + `delete_node`
    + `insert_attr`
    + `delete_attr`
    + `is_empty`
    + `has_node`
    + `has_nodes`
    + `node_count`
    + `has_attr`
    + `has_attrs`
    + `attr_count`
- `types::Document`
    + `insert_node`
    + `delete_node`
    + `is_empty`
    + `has_node`
    + `has_nodes`
    + `node_count`

***

# 0.1.0 (2015-09-26)

Initial release
