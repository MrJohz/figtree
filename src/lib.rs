//! A library to parse and work with figtree documents.
//!
//! Figtree is a file format designed for config files that need to be easily manipulated
//! by real life humans.  It is made up of nodes, keys, and values, and looks a bit like
//! this:
//!
//! ```text
//! node {
//!     "key": "value",
//!     "multiple types": ["strings", 1, 2.0, false, !identifier],
//!
//!     subnodes {
//!         "with": {"more": "key", "value": "pairs"}
//!     }
//! }
//! ```
//!
//! The figtree library parses structures like this into documents that can be
//! manipulated to use as an efficient configuration system.
//!
//! # Examples
//! ```
//! extern crate figtree;
//!
//! let mut figgy = figtree::Figtree::from_string("node { 'key': 'val' }");
//! let config = figgy.parse().ok().expect("parsing error");
//! let value = config.get_node("node")
//!     .and_then(|node| node.get_attr("key"))
//!     .and_then(|value| value.get_str())
//!     .expect("could not obtain value");
//! assert!(value == "val");
//! ```

#[macro_use]
extern crate matches;

mod utils;

mod position;
pub use position::Position;

mod lexer;
pub use lexer::LexToken;
pub use lexer::LexError;

mod parser;
pub use parser::ParseError;

pub mod types;
pub use types::*;

mod figtree;
pub use figtree::Figtree;
