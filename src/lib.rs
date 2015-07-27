#![feature(unboxed_closures, plugin, rc_weak)]
#![plugin(string_cache_plugin)]

extern crate html5ever;
#[macro_use] extern crate matches;
extern crate selectors;
extern crate string_cache;
extern crate tendril;
#[cfg(test)] extern crate tempdir;

pub use parser::{Html, ParseOpts};
pub use select::{Selectors, Select};

pub mod tree;

mod parser;
mod select;
mod serializer;
mod cell_option;

#[cfg(test)] mod tests;
