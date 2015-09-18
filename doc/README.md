% Figtree

# Figtree

Figtree is a file-format designed for configuration files.  It allows for multiple,
nested sections, many data-types included dict and list container types, and attempts
to be expressive and useable for human maintainers.

A figtree file (`.ft`) looks a bit like this:

```javascript
// C-style inline comments and block comments are allowed
/* block comments /* can be */ nested */
myconfig {
    // Nodes consist of an identifier followed by a brace-block.
    // Each node can have a number of key-value attributes attached.
    // Keys must be strings, but values can be any type.

    identifiers {
        // Figtree uses Swift's rules for identifiers, which means that emojii are
        // allowed.  Additionally, `quoted identifiers` can also be used if an identifier
        // is wanted that contains disallowed characters

        🐶 { /* emojii dog */ }
        underscores_allowed {}
        numbers_allowed_345 { /* except as the first character */ }
    }

    value_types {
        // strings can be written with single or double quotes
        // multiple string literals in a row will be concatenated together
        // strings can span multiple lines
        "strings": "with single" ' or double ' "quotes"
        // this is equivalent to "with single or double quotes"

        // integers can be written using standard numerals
        // or by prefixing with 0[xdob] for hexadecimal, decimal, octal, or binary
        "integers": [+34, -42, 0x4f, 0d34, 0o42, 0b1010]

        // floats can only be written in decimal
        // floats can have exponents using either 'e' or 'E'
        "floats": [3.4, .5, 8.e4, -4.5, +4.5E4]

        // booleans are either 'true' or 'false'
        "booleans": [true, false] // really not much else here...

        // identifiers can be used as values by prefixing with an exclamation point (!)
        // idents can be used much like strings, but can be given special meaning
        "identifiers": [!ident, !false, !`quoted using \` characters`]

        // lists are pairs of brackets that contain comma-separated values
        // trailing commas are allowed
        "lists": [["nested"], [["lists"], "are"], "allowed",]

        // dicts are pairs of braces that contain comma-separated key-value pairs
        // keys must always be strings
        // trailing commas are allowed
        "dicts": { "dicts": { "can": !also }, "be": "nested" }
    }
}
