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

mod attributes;
#[cfg(feature = "hyper")] mod hyper;
pub mod iter;
mod move_cell;
mod node_data_ref;
mod parser;
mod select;
mod serializer;
#[cfg(test)] mod tests;
mod tree;

pub use attributes::Attributes;
pub use node_data_ref::NodeDataRef;
pub use parser::{parse_html, ParseOpts};
pub use select::Selectors;
pub use tree::{NodeRef, Node, NodeData, ElementData, Doctype, DocumentData};

/// This module re-exports a number of traits that are useful when using Kuchiki.
/// It can be used with:
///
/// ```rust
/// use kuchiki::traits::*;
/// ```
pub mod traits {
    pub use html5ever::tendril::TendrilSink;
    pub use iter::{NodeIterator, ElementIterator};
    #[cfg(feature = "hyper")] pub use hyper::{ParserExt, IntoResponse};
}

