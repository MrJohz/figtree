[![Build Status](https://travis-ci.org/MrJohz/figtree.svg?branch=master)](https://travis-ci.org/MrJohz/figtree)

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

## Comparison with other formats

### .ini/.cfg

- Ini sections are pretty much directly translatable into Figtree nodes.  Most
    ini parsers will allow a wider range of characters in section headers, whereas
    Figtree has a more restrictive format.  Figtree sections naturally nest - this can be
    replicated in ini configs, usually using a '.' path format, but has to be defined by
    the parser

- Ini datatypes are generally fairly simple - often just strings, although some parsers
    may make a distinction between different primitives.  Figtree allows strings,
    integers, floats and booleans, as well as mixed collections of those kinds.

- There is no universal ini specification.  Figtree has one specification, meaning that
    parsers can be written for it in multiple languages and they will all understand the
    same set of documents.  It also uses familiar JSON-like datatypes where possible,
    meaning that what users expect to work probably will work.


### JSON

- JSON makes no distinction between nested objects and nested sections.  For example,
    `{'a': {'b': {'c': true}}}` could be setting the value of setting `a.b.c` to `true`,
    or it could be setting the value of setting `a.b` to `{'c': true}`, or it could be
    setting the value of `a` to `{'b': {'c': true}}`.  This means that complex nested
    JSON config files can be difficult to decipher if one doesn't have a clear
    specification to hand.

    Figtree, however, differentiates between nested sections - defined using an
    `ident {}` format - and nested objects - defined using a `{'key': 'value'}` format.
    This provides semantic meaning to a figtree document outside of the meaning given by
    the application-specific parser.

- The JSON spec is often criticised as being too strict.  There are no comments, and no
    trailing commas.  Strings must be quoted with double-quotes, and the spec leaves no
    room for hexadecimal, octal, or binary literals.  Indeed, many JSON users are
    encouraged to pass JSON documents through a Javascript minifier before reading them,
    to remove comments and simplify the document.

    Figtree maintains a certain sense of JSON's strictness, but it makes allowances for
    all of these areas - two kinds of comment are allowed, trailing commas are allowed
    but optional, both single- and double-quotes are allowed for strings, and the
    numeric parser accepts integer literals in a range of formats to ensure the user is
    able to write their config file in almost any way they see fit.

- JSON officially deals with all numbers as "numeric", just like Javascript.  Figtree
    internally treats floats and integers as separate types, although in practice a
    developer using Figtree in Javascript will probably treat all numerics equally, while
    a developer using JSON in Rust will probably treat integers and floats separately, so
    this distinction may not exist in practice.

- JSON is a subset of the Javascript language, which means any Javascript parser can
    read it.  Figtree is not, nor is it a subset of any language or syntax that I am
    aware of, and thus needs a specialised parser to read.  JSON also has many parsers
    in different languages.  I also only know of one Figtree parser - this one.  This is
    a known bug...


### YAML

- YAML is designed to be human-useable, which makes it a very good configuration tool.
    Figtree is also designed with human use in mind.  However, the YAML parser is
    generally more lenient than the Figtree parser - in most circumstances, YAML will
    read unquoted text as strings.  This can cause issues as the user can assume unquoted
    strings will always work then suddenly end up with a strange or confusing bug.  In
    Figtree all strings must be quoted.  I do not believe there is any instance where a
    piece of Figtree data can change value, and that change accidentally change the type
    of that data.

- YAML will accept any datatype as a key.  This means that keys should be quoted, unless
    you want to suddenly have keys of `true` and `false` instead of `"Yes"` or `"No"`.
    (This has happened to me before.  It is irritating.)  Figtree keys must always be
    strings.  This is more for convenience than anything else - the current parser is
    written in Rust, which has static types, and assuming all the keys will be strings
    makes it easier to build type-safe datastructures.

- Like JSON, YAML deals with maps and lists of values, and has no concept of sections.
    While a YAML user may write a section using one syntax and a map using another
    syntax, the parser will treat both of these in the same way.

- The YAML specification allows implementers to arbitrarily load any class or structure
    (or indeed call any function) that the implementation language has access to.  This
    has famously lead to security errors in large libraries and programs, including Ruby
    on Rails.  Figtree does not allow the creation of arbitrary language objects through
    clever control sequences.  Developers may choose to automatically create language
    objects based on particular sequences of Figtree structures, but they must choose to
    turn it on, and define what those structures are, and define what the language
    objects are.

- YAML has syntactically significant whitespace.  There is nothing wrong with this.  I
    like developing with Python, it's fun.  Figtree probably uses braces because I was
    using a braced language at the time I decided I wanted to write it.  You can go and
    write your own version of Figtree that uses syntactically significant whitespace if
    you want, in fact I would love to see what that looked like.  Please don't hurt me.


### TOML

- Much like ini files, TOML has a concept of sections.  However, in TOML those sections
    can be directly translated into nested objects - any valid TOML document can be
    converted into a valid JSON document, and vice versa.  While users of a TOML config
    file may assign different meanings to section headings and dictionary literals, to
    the developer using a TOML library, these two syntaxes mean exactly the same thing.
    Figtree always has a semantic difference between sections and key-value pairs.

- TOML arrays may not contain mixed types.  Figtree arrays can contain a mix of different
    types.  In reality, Figtree arrays almost certainly should be uniformly typed, but
    that is left to the developer to enforce.

- TOML has semi-syntactically significant whitespace.  Some portions of the spec act like
    line-based formats, although multiline strings and arrays would probably break most
    simple line-by-line parsers.  In particular, dictionary literals may not contain
    newlines.  Figtree laughs at this madness, and allows you to put whitespace
    everywhere, or nowhere.


## TODO:
- Lexing:
    + Better unicode escapes
    + Other booleans?
- Parsing:
    + Advanced position details - show beginning and end of token
    + Interpolation via `$reference` nodes?
- API Features
    + Write files out
    + Sugar functions for manipulating configuration structs
    + Integrate serialisation/deserialisation
    + Pull parser API?
