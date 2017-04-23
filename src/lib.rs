/*!

Kuchiki (朽木), a HTML/XML tree manipulation library for Rust.

*/

#![deny(missing_docs)]

extern crate cssparser;
extern crate html5ever;
#[macro_use] extern crate html5ever_atoms;
#[macro_use] extern crate matches;
extern crate ref_slice;
extern crate selectors;
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
pub use parser::{parse_html, parse_html_with_options, ParseOpts};
pub use select::{Selectors, Selector, Specificity};
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

