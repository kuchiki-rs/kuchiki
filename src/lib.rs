#![feature(unboxed_closures, core, plugin)]
#![plugin(string_cache_plugin)]

extern crate html5ever;
#[macro_use] extern crate matches;
extern crate selectors;
extern crate string_cache;
extern crate typed_arena;

pub use parser::{parse, ParseOpts};

pub mod tree;

mod parser;
mod select;
mod serializer;
