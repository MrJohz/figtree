# Figtree - the noded config file

## What is Figtree?

Figtree is a data format designed mainly for configuration with nodes and sections.  It
looks a bit like this:

```javascript
// C-style inline comments and block comments are allowed
myconfig {  // nodes consist of an identifier followed by a brace-block
    // each node can have a number of key-value attributes attached.
    // keys must be strings, but values can be strings,
    // lists, ints, floats or bools
    "key": "value",
    "list": ["one", 2, "three"],
    "trailing comma?": true,

    // nodes can also contain sub-nodes
    empty_subnode {
        /* empty subnode */
    }

    just_subnodes {
        subsubnode {}
    }

    "more": {"key": "value", "pairs": "allowed"}
}
```

## Why Figtree?

Figtree is brilliant for configuration files.  Configuration is often scoped - a "server"
section may contain details about the server configuration, whilst a "client" will change
how the client works, and a "database" section provides details for setting up database
connections.  In Figtree, each of those sections can be represented as a node, and the
actual configuration as the key-value pairs within that scope.

Figtree also attempts to be human-readable/writeable.  The grammar allows for two kinds
of comments, trailing commas, and a wide variety of forms for different literals.  This
means that Figtree files can be read and modified by users easily, making ideal for
configuration files.
