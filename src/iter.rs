//! Node iterators

use std::cell::RefCell;
use std::iter::Rev;

use tree::{NodeRef, NodeDataRef, ElementData};
use select::{Selectors, Select};

impl NodeRef {
    /// Return an iterator of references to this node and its ancestors.
    pub fn inclusive_ancestors(&self) -> Ancestors {
        Ancestors(Some(self.clone()))
    }

    /// Return an iterator of references to this node’s ancestors.
    pub fn ancestors(&self) -> Ancestors {
        Ancestors(self.parent())
    }

    /// Return an iterator of references to this node and the siblings before it.
    pub fn inclusive_preceding_siblings(&self) -> Rev<Siblings> {
        match self.parent() {
            Some(parent) => {
                let first_sibling = parent.first_child().unwrap();
                debug_assert!(self.previous_sibling().is_some() || *self == first_sibling);
                Siblings(Some(State { next: first_sibling, next_back: self.clone() }))
            }
            None => {
                debug_assert!(self.previous_sibling().is_none());
                Siblings(Some(State { next: self.clone(), next_back: self.clone() }))
            }
        }.rev()
    }

    /// Return an iterator of references to this node’s siblings before it.
    pub fn preceding_siblings(&self) -> Rev<Siblings> {
        match (self.parent(), self.previous_sibling()) {
            (Some(parent), Some(previous_sibling)) => {
                let first_sibling = parent.first_child().unwrap();
                Siblings(Some(State { next: first_sibling, next_back: previous_sibling }))
            }
            _ => Siblings(None)
        }.rev()
    }

    /// Return an iterator of references to this node and the siblings after it.
    pub fn inclusive_following_siblings(&self) -> Siblings {
        match self.parent() {
            Some(parent) => {
                let last_sibling = parent.last_child().unwrap();
                debug_assert!(self.next_sibling().is_some() || *self == last_sibling);
                Siblings(Some(State { next: self.clone(), next_back: last_sibling }))
            }
            None => {
                debug_assert!(self.next_sibling().is_none());
                Siblings(Some(State { next: self.clone(), next_back: self.clone() }))
            }
        }
    }

    /// Return an iterator of references to this node’s siblings after it.
    pub fn following_siblings(&self) -> Siblings {
        match (self.parent(), self.next_sibling()) {
            (Some(parent), Some(next_sibling)) => {
                let last_sibling = parent.last_child().unwrap();
                Siblings(Some(State { next: next_sibling, next_back: last_sibling }))
            }
            _ => Siblings(None)
        }
    }

    /// Return an iterator of references to this node’s children.
    pub fn children(&self) -> Siblings {
        match (self.first_child(), self.last_child()) {
            (Some(first_child), Some(last_child)) => {
                Siblings(Some(State { next: first_child, next_back: last_child }))
            }
            (None, None) => Siblings(None),
            _ => unreachable!()
        }
    }

    /// Return an iterator of references to this node and its descendants, in tree order.
    ///
    /// Parent nodes appear before the descendants.
    ///
    /// Note: this is the `NodeEdge::Start` items from `traverse()`.
    pub fn inclusive_descendants(&self) -> Descendants {
        Descendants(self.traverse_inclusive())
    }

    /// Return an iterator of references to this node’s descendants, in tree order.
    ///
    /// Parent nodes appear before the descendants.
    ///
    /// Note: this is the `NodeEdge::Start` items from `traverse()`.
    pub fn descendants(&self) -> Descendants {
        Descendants(self.traverse())
    }

    /// Return an iterator of the start and end edges of this node and its descendants,
    /// in tree order.
    pub fn traverse_inclusive(&self) -> Traverse {
        Traverse(Some(State {
            next: NodeEdge::Start(self.clone()),
            next_back: NodeEdge::End(self.clone()),
        }))
    }

    /// Return an iterator of the start and end edges of this node’s descendants,
    /// in tree order.
    pub fn traverse(&self) -> Traverse {
        match (self.first_child(), self.last_child()) {
            (Some(first_child), Some(last_child)) => {
                Traverse(Some(State {
                    next: NodeEdge::Start(first_child),
                    next_back: NodeEdge::End(last_child)
                }))
            }
            (None, None) => Traverse(None),
            _ => unreachable!()
        }
    }

    pub fn select(&self, selectors: &str) -> Result<Select<Elements<Descendants>>, ()> {
        self.inclusive_descendants().select(selectors)
    }
}


#[derive(Debug, Clone)]
struct State<T> {
    next: T,
    next_back: T,
}


/// A double-ended iterator of sibling nodes.
#[derive(Debug, Clone)]
pub struct Siblings(Option<State<NodeRef>>);

macro_rules! siblings_next {
    ($next: ident, $next_back: ident, $next_sibling: ident) => {
        fn $next(&mut self) -> Option<NodeRef> {
            #![allow(non_shorthand_field_patterns)]
            self.0.take().map(|State { $next: next, $next_back: next_back }| {
                if let Some(sibling) = next.$next_sibling() {
                    if next != next_back {
                        self.0 = Some(State { $next: sibling, $next_back: next_back })
                    }
                }
                next
            })
        }
    }
}

impl Iterator for Siblings {
    type Item = NodeRef;
    siblings_next!(next, next_back, next_sibling);
}

impl DoubleEndedIterator for Siblings {
    siblings_next!(next_back, next, previous_sibling);
}


/// An iterator on ancestor nodes.
#[derive(Debug, Clone)]
pub struct Ancestors(Option<NodeRef>);

impl Iterator for Ancestors {
    type Item = NodeRef;

    fn next(&mut self) -> Option<NodeRef> {
        self.0.take().map(|node| {
            self.0 = node.parent();
            node
        })
    }
}


/// An iterator of references to a given node and its descendants, in tree order.
#[derive(Debug, Clone)]
pub struct Descendants(Traverse);

macro_rules! descendants_next {
    ($next: ident) => {
        fn $next(&mut self) -> Option<NodeRef> {
            loop {
                match (self.0).$next() {
                    Some(NodeEdge::Start(node)) => return Some(node),
                    Some(NodeEdge::End(_)) => {}
                    None => return None
                }
            }
        }
    }
}

impl Iterator for Descendants {
    type Item = NodeRef;
    descendants_next!(next);
}

impl DoubleEndedIterator for Descendants {
    descendants_next!(next_back);
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeEdge<T> {
    /// Indicates that start of a node that has children.
    /// Yielded by `Traverse::next` before the node’s descendants.
    /// In HTML or XML, this corresponds to an opening tag like `<div>`
    Start(T),

    /// Indicates that end of a node that has children.
    /// Yielded by `Traverse::next` after the node’s descendants.
    /// In HTML or XML, this corresponds to a closing tag like `</div>`
    End(T),
}


/// An iterator of the start and end edges of the nodes in a given subtree.
#[derive(Debug, Clone)]
pub struct Traverse(Option<State<NodeEdge<NodeRef>>>);

macro_rules! traverse_next {
    ($next: ident, $next_back: ident, $first_child: ident, $next_sibling: ident, $Start: ident, $End: ident) => {
        fn $next(&mut self) -> Option<NodeEdge<NodeRef>> {
            #![allow(non_shorthand_field_patterns)]
            self.0.take().map(|State { $next: next, $next_back: next_back }| {
                if next != next_back {
                    self.0 = match next {
                        NodeEdge::$Start(ref node) => {
                            match node.$first_child() {
                                Some(child) => {
                                    Some(State { $next: NodeEdge::$Start(child), $next_back: next_back })
                                }
                                None => Some(State { $next: NodeEdge::$End(node.clone()), $next_back: next_back })
                            }
                        }
                        NodeEdge::$End(ref node) => {
                            match node.$next_sibling() {
                                Some(sibling) => {
                                    Some(State { $next: NodeEdge::$Start(sibling), $next_back: next_back })
                                }
                                None => node.parent().map(|parent| {
                                    State { $next: NodeEdge::$End(parent), $next_back: next_back }
                                })
                            }
                        }
                    };
                }
                next
            })
        }
    }
}

impl Iterator for Traverse {
    type Item = NodeEdge<NodeRef>;
    traverse_next!(next, next_back, first_child, next_sibling, Start, End);
}

impl DoubleEndedIterator for Traverse {
    traverse_next!(next_back, next, last_child, previous_sibling, End, Start);
}


macro_rules! filter_map_like_iterator {
    ($name: ident, $f: expr, $from: ty, $to: ty) => {
        #[derive(Debug, Clone)]
        pub struct $name<I>(pub I);

        impl<I> Iterator for $name<I> where I: Iterator<Item=$from> {
            type Item = $to;

            #[inline]
            fn next(&mut self) -> Option<$to> {
                for x in self.0.by_ref() {
                    if let Some(y) = ($f)(x) {
                        return Some(y)
                    }
                }
                None
            }
        }

        impl<I> DoubleEndedIterator for $name<I> where I: DoubleEndedIterator<Item=$from> {
            #[inline]
            fn next_back(&mut self) -> Option<$to> {
                for x in self.0.by_ref().rev() {
                    if let Some(y) = ($f)(x) {
                        return Some(y)
                    }
                }
                None
            }
        }
    }
}

filter_map_like_iterator!(Elements, NodeRef::into_element_ref, NodeRef, NodeDataRef<ElementData>);
filter_map_like_iterator!(Comments, NodeRef::into_comment_ref, NodeRef, NodeDataRef<RefCell<String>>);
filter_map_like_iterator!(TextNodes, NodeRef::into_text_ref, NodeRef, NodeDataRef<RefCell<String>>);

pub trait NodeIterator: Sized + Iterator<Item=NodeRef> {
    fn elements(self) -> Elements<Self> {
        Elements(self)
    }
    fn text_nodes(self) -> TextNodes<Self> {
        TextNodes(self)
    }
    fn comments(self) -> Comments<Self> {
        Comments(self)
    }
    fn select(self, selectors: &str) -> Result<Select<Elements<Self>>, ()> {
        self.elements().select(selectors)
    }
}


pub trait ElementIterator: Sized + Iterator<Item=NodeDataRef<ElementData>> {
    fn select(self, selectors: &str) -> Result<Select<Self>, ()> {
        Selectors::compile(selectors).map(|s| s.filter(self))
    }
}

impl<I> NodeIterator for I where I: Iterator<Item=NodeRef> {}
impl<I> ElementIterator for I where I: Iterator<Item=NodeDataRef<ElementData>> {}
