/*!

Kuchiki (朽木), a HTML/XML tree manipulation library for Rust.

*/

#![cfg_attr(feature = "unstable", feature(rc_weak, rc_counts))]
#![deny(missing_docs)]

extern crate html5ever;
#[macro_use] extern crate matches;
extern crate selectors;
extern crate rc;
#[macro_use] extern crate string_cache;
#[cfg(test)] extern crate tempdir;
extern crate tendril;

pub mod iter;
mod move_cell;
mod node_data_ref;
mod parser;
mod select;
mod serializer;
#[cfg(test)] mod tests;
mod tree;

pub use iter::{NodeIterator, ElementIterator};
pub use node_data_ref::NodeDataRef;
pub use parser::{Html, ParseOpts};
pub use select::Selectors;
pub use tree::{NodeRef, Node, NodeData, ElementData, Doctype, DocumentData};
