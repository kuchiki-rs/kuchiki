pub use arena_tree::{Ref, RefMut, Ancestors, PrecedingSiblings, FollowingSiblings,
                     Children, ReverseChildren, Descendants, Traverse, ReverseTraverse, NodeEdge};
pub use string_cache::{Atom, Namespace, QualName};

use arena_tree;
use std::collections::HashMap;


pub type NodeRef<'a> = arena_tree::NodeRef<'a, NodeData>;
pub type Arena<'a> = arena_tree::Arena<'a, NodeData>;

#[derive(Debug)]
pub enum NodeData {
    Element(ElementData),
    Text(String),
    Comment(String),
    Doctype(Doctype),
    Document,
}

#[derive(Debug)]
pub struct Doctype {
    pub name: String,
    pub public_id: String,
    pub system_id: String,
}

#[derive(Debug)]
pub struct ElementData {
    pub name: QualName,
    pub attributes: HashMap<QualName, String>,
}
