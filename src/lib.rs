#![feature(unboxed_closures, core, plugin, alloc)]
#![plugin(string_cache_plugin)]

extern crate html5ever;
#[macro_use] extern crate matches;
extern crate movecell;
extern crate selectors;
extern crate string_cache;

pub use parser::{Html, ParseOpts};

pub mod tree;

mod parser;
mod select;
mod serializer;

#[cfg(test)] mod tests;
