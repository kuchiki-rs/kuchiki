#![cfg_attr(feature = "unstable", feature(rc_weak, rc_counts, plugin))]
#![cfg_attr(feature = "unstable", plugin(string_cache_plugin))]

extern crate html5ever;
#[macro_use] extern crate matches;
extern crate selectors;
extern crate rc;
#[macro_use] extern crate string_cache;
extern crate tendril;
#[cfg(test)] extern crate tempdir;

pub use parser::{Html, ParseOpts};
pub use select::{Selectors, Select};
pub use tree::NodeRef;

pub mod tree;

mod parser;
mod select;
mod serializer;
mod cell_option;
mod iter;

#[cfg(test)] mod tests;
