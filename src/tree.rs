use html5ever::tree_builder::QuirksMode;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::iter::Rev;
use string_cache::QualName;
use typed_arena::Arena;


#[derive(Debug, PartialEq, Clone)]
pub enum NodeData {
    Element(ElementData),
    Text(RefCell<String>),
    Comment(RefCell<String>),
    Doctype(Doctype),
    Document(DocumentData),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Doctype {
    pub name: String,
    pub public_id: String,
    pub system_id: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ElementData {
    pub name: QualName,
    pub attributes: RefCell<HashMap<QualName, String>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DocumentData {
    pub _quirks_mode: Cell<QuirksMode>,
}

impl DocumentData {
    pub fn quirks_mode(&self) -> QuirksMode {
        self._quirks_mode.get()
    }
}

/// A node inside a DOM-like tree.
pub struct Node<'a> {
    parent: Cell<Option<&'a Node<'a>>>,
    previous_sibling: Cell<Option<&'a Node<'a>>>,
    next_sibling: Cell<Option<&'a Node<'a>>>,
    first_child: Cell<Option<&'a Node<'a>>>,
    last_child: Cell<Option<&'a Node<'a>>>,
    pub data: NodeData,
}


impl<'a> Eq for Node<'a> {}
impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Node<'a>) -> bool {
        self as *const Node<'a> == other as *const Node<'a>
    }
}

trait Take<T> {
    fn take(&self) -> Option<T>;
}

impl<T: Copy> Take<T> for Cell<Option<T>> {
    fn take(&self) -> Option<T> {
        let value = self.get();
        self.set(None);
        value
    }
}

impl<'a> Node<'a> {
    /// Create a new node from its associated data.
    pub fn new(data: NodeData, arena: &'a Arena<Node<'a>>) -> &'a Node<'a> {
        arena.alloc(Node {
            parent: Cell::new(None),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
            previous_sibling: Cell::new(None),
            next_sibling: Cell::new(None),
            data: data,
        })
    }

    pub fn new_element<I>(name: QualName, attributes: I, arena: &'a Arena<Node<'a>>)
                          -> &'a Node<'a>
                          where I: IntoIterator<Item=(QualName, String)> {
        Node::new(NodeData::Element(ElementData {
            name: name,
            attributes: RefCell::new(attributes.into_iter().collect()),
        }), arena)
    }

    pub fn new_text<T: Into<String>>(value: T, arena: &'a Arena<Node<'a>>) -> &'a Node<'a> {
        Node::new(NodeData::Text(RefCell::new(value.into())), arena)
    }

    pub fn new_comment<T: Into<String>>(value: T, arena: &'a Arena<Node<'a>>) -> &'a Node<'a> {
        Node::new(NodeData::Comment(RefCell::new(value.into())), arena)
    }

    pub fn new_doctype<T1, T2, T3>(name: T1, public_id: T2, system_id: T3,
                                   arena: &'a Arena<Node<'a>>)
                                   -> &'a Node<'a>
                                   where T1: Into<String>, T2: Into<String>, T3: Into<String> {
        Node::new(NodeData::Doctype(Doctype {
            name: name.into(),
            public_id: public_id.into(),
            system_id: system_id.into(),
        }), arena)
    }

    pub fn new_document(arena: &'a Arena<Node<'a>>) -> &'a Node<'a> {
        Node::new(NodeData::Document(DocumentData {
            _quirks_mode: Cell::new(QuirksMode::NoQuirks),
        }), arena)
    }

    pub fn as_element(&self) -> Option<&ElementData> {
        match self.data {
            NodeData::Element(ref value) => Some(value),
            _ => None
        }
    }

    pub fn as_text(&self) -> Option<&RefCell<String>> {
        match self.data {
            NodeData::Text(ref value) => Some(value),
            _ => None
        }
    }

    pub fn as_comment(&self) -> Option<&RefCell<String>> {
        match self.data {
            NodeData::Comment(ref value) => Some(value),
            _ => None
        }
    }

    pub fn as_doctype(&self) -> Option<&Doctype> {
        match self.data {
            NodeData::Doctype(ref value) => Some(value),
            _ => None
        }
    }

    pub fn as_document(&self) -> Option<&DocumentData> {
        match self.data {
            NodeData::Document(ref value) => Some(value),
            _ => None
        }
    }

    /// Return a reference to the parent node, unless this node is the root of the tree.
    pub fn parent(&self) -> Option<&'a Node<'a>> {
        self.parent.get()
    }

    /// Return a reference to the first child of this node, unless it has no child.
    pub fn first_child(&self) -> Option<&'a Node<'a>> {
        self.first_child.get()
    }

    /// Return a reference to the last child of this node, unless it has no child.
    pub fn last_child(&self) -> Option<&'a Node<'a>> {
        self.last_child.get()
    }

    /// Return a reference to the previous sibling of this node, unless it is a first child.
    pub fn previous_sibling(&self) -> Option<&'a Node<'a>> {
        self.previous_sibling.get()
    }

    /// Return a reference to the previous sibling of this node, unless it is a last child.
    pub fn next_sibling(&self) -> Option<&'a Node<'a>> {
        self.next_sibling.get()
    }

    /// Return an iterator of references to this node and its ancestors.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn ancestors(&'a self) -> Ancestors<'a> {
        Ancestors(Some(self))
    }

    /// Return an iterator of references to this node and the siblings before it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn preceding_siblings(&'a self) -> Rev<Siblings<'a>> {
        match self.parent() {
            Some(parent) => {
                let first_sibling = parent.first_child().unwrap();
                debug_assert!(self.previous_sibling().is_some() || self == first_sibling);
                Siblings(Some((first_sibling, self)))
            }
            None => {
                debug_assert!(self.previous_sibling().is_none());
                Siblings(Some((self, self)))
            }
        }.rev()
    }

    /// Return an iterator of references to this node and the siblings after it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn following_siblings(&'a self) -> Siblings<'a> {
        match self.parent() {
            Some(parent) => {
                let last_sibling = parent.last_child().unwrap();
                debug_assert!(self.next_sibling().is_some() || self == last_sibling);
                Siblings(Some((self, last_sibling)))
            }
            None => {
                debug_assert!(self.next_sibling().is_none());
                Siblings(Some((self, self)))
            }
        }
    }

    /// Return an iterator of references to this node’s children.
    pub fn children(&self) -> Siblings<'a> {
        match (self.first_child(), self.last_child()) {
            (Some(first_child), Some(last_child)) => Siblings(Some((first_child, last_child))),
            (None, None) => Siblings(None),
            _ => unreachable!()
        }
    }

    /// Return an iterator of references to this node and its descendants, in tree order.
    ///
    /// Parent nodes appear before the descendants.
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    ///
    /// Note: this is the `NodeEdge::Start` items from `traverse()`.
    pub fn descendants(&'a self) -> Descendants<'a> {
        Descendants(self.traverse())
    }

    /// Return an iterator of the start and end edges of this node and its descendants,
    /// in tree order.
    pub fn traverse(&'a self) -> Traverse<'a> {
        Traverse(Some((NodeEdge::Start(self), NodeEdge::End(self))))
    }

    /// Detach a node from its parent and siblings. Children are not affected.
    pub fn detach(&self) {
        let parent = self.parent.take();
        let previous_sibling = self.previous_sibling.take();
        let next_sibling = self.next_sibling.take();

        if let Some(next_sibling) = next_sibling {
            next_sibling.previous_sibling.set(previous_sibling);
        } else if let Some(parent) = parent {
            parent.last_child.set(previous_sibling);
        }

        if let Some(previous_sibling) = previous_sibling {
            previous_sibling.next_sibling.set(next_sibling);
        } else if let Some(parent) = parent {
            parent.first_child.set(next_sibling);
        }
    }

    /// Append a new child to this node, after existing children.
    pub fn append(&'a self, new_child: &'a Node<'a>) {
        new_child.detach();
        new_child.parent.set(Some(self));
        if let Some(last_child) = self.last_child.take() {
            new_child.previous_sibling.set(Some(last_child));
            debug_assert!(last_child.next_sibling().is_none());
            last_child.next_sibling.set(Some(new_child));
        } else {
            debug_assert!(self.first_child().is_none());
            self.first_child.set(Some(new_child));
        }
        self.last_child.set(Some(new_child));
    }

    /// Prepend a new child to this node, before existing children.
    pub fn prepend(&'a self, new_child: &'a Node<'a>) {
        new_child.detach();
        new_child.parent.set(Some(self));
        if let Some(first_child) = self.first_child.take() {
            debug_assert!(first_child.previous_sibling().is_none());
            first_child.previous_sibling.set(Some(new_child));
            new_child.next_sibling.set(Some(first_child));
        } else {
            debug_assert!(self.first_child().is_none());
            self.last_child.set(Some(new_child));
        }
        self.first_child.set(Some(new_child));
    }

    /// Insert a new sibling after this node.
    pub fn insert_after(&'a self, new_sibling: &'a Node<'a>) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent());
        new_sibling.previous_sibling.set(Some(self));
        if let Some(next_sibling) = self.next_sibling.take() {
            debug_assert!(next_sibling.previous_sibling().unwrap() == self);
            next_sibling.previous_sibling.set(Some(new_sibling));
            new_sibling.next_sibling.set(Some(next_sibling));
        } else if let Some(parent) = self.parent() {
            debug_assert!(parent.last_child().unwrap() == self);
            parent.last_child.set(Some(new_sibling));
        }
        self.next_sibling.set(Some(new_sibling));
    }

    /// Insert a new sibling before this node.
    pub fn insert_before(&'a self, new_sibling: &'a Node<'a>) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent());
        new_sibling.next_sibling.set(Some(self));
        if let Some(previous_sibling) = self.previous_sibling.take() {
            new_sibling.previous_sibling.set(Some(previous_sibling));
            debug_assert!(previous_sibling.next_sibling().unwrap() == self);
            previous_sibling.next_sibling.set(Some(new_sibling));
        } else if let Some(parent) = self.parent() {
            debug_assert!(parent.first_child().unwrap() == self);
            parent.first_child.set(Some(new_sibling));
        }
        self.previous_sibling.set(Some(new_sibling));
    }
}


/// A double-ended iterator of sibling nodes.
pub struct Siblings<'a>(Option<(&'a Node<'a>, &'a Node<'a>)>);

impl<'a> Iterator for Siblings<'a> {
    type Item = &'a Node<'a>;

    fn next(&mut self) -> Option<&'a Node<'a>> {
        self.0.take().map(|(next, next_back)| {
            if let Some(sibling) = next.next_sibling() {
                if next != next_back {
                    self.0 = Some((sibling, next_back))
                }
            }
            next
        })
    }
}

impl<'a> DoubleEndedIterator for Siblings<'a> {
    fn next_back(&mut self) -> Option<&'a Node<'a>> {
        self.0.map(|(next, next_back)| {
            self.0 = match next_back.previous_sibling() {
                Some(sibling) if next != next_back => Some((next, sibling)),
                _ => None
            };
            next_back
        })
    }
}


/// An iterator on ancestor nodes.
pub struct Ancestors<'a>(Option<&'a Node<'a>>);

impl<'a> Iterator for Ancestors<'a> {
    type Item = &'a Node<'a>;

    fn next(&mut self) -> Option<&'a Node<'a>> {
        self.0.map(|node| {
            self.0 = node.parent();
            node
        })
    }
}


/// An iterator of references to a given node and its descendants, in tree order.
pub struct Descendants<'a>(Traverse<'a>);

impl<'a> Iterator for Descendants<'a> {
    type Item = &'a Node<'a>;

    fn next(&mut self) -> Option<&'a Node<'a>> {
        loop {
            match self.0.next() {
                Some(NodeEdge::Start(node)) => return Some(node),
                Some(NodeEdge::End(_)) => {}
                None => return None
            }
        }
    }
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
pub struct Traverse<'a>(Option<(NodeEdge<&'a Node<'a>>, NodeEdge<&'a Node<'a>>)>);

impl<'a> Iterator for Traverse<'a> {
    type Item = NodeEdge<&'a Node<'a>>;

    fn next(&mut self) -> Option<NodeEdge<&'a Node<'a>>> {
        self.0.map(|(next, next_back)| {
            self.0 = if next == next_back {
                None
            } else {
                match next {
                    NodeEdge::Start(node) => {
                        match node.first_child() {
                            Some(child) => Some((NodeEdge::Start(child), next_back)),
                            None => Some((NodeEdge::End(node), next_back))
                        }
                    }
                    NodeEdge::End(node) => {
                        match node.next_sibling() {
                            Some(sibling) => Some((NodeEdge::Start(sibling), next_back)),
                            None => node.parent().map(|parent| (NodeEdge::End(parent), next_back))
                        }
                    }
                }
            };
            next
        })
    }
}

impl<'a> DoubleEndedIterator for Traverse<'a> {
    fn next_back(&mut self) -> Option<NodeEdge<&'a Node<'a>>> {
        self.0.map(|(next, next_back)| {
            self.0 = if next == next_back {
                None
            } else {
                match next_back {
                    NodeEdge::End(node) => {
                        match node.last_child() {
                            Some(child) => Some((next, NodeEdge::End(child))),
                            None => Some((next, NodeEdge::Start(node)))
                        }
                    }
                    NodeEdge::Start(node) => {
                        match node.previous_sibling() {
                            Some(sibling) => Some((next, NodeEdge::End(sibling))),
                            None => node.parent().map(|parent| (next, NodeEdge::Start(parent)))
                        }
                    }
                }
            };
            next
        })
    }
}
