#![feature(unboxed_closures, core)]

extern crate html5ever;
extern crate string_cache;
extern crate typed_arena;

pub use parser::{parse, ParseOpts};

pub mod tree;

mod parser;
mod serializer;

#[cfg(test)] mod tests;
