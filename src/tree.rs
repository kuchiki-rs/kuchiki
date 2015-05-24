use html5ever::tree_builder::QuirksMode;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use string_cache::QualName;


#[derive(Debug, PartialEq, Clone)]
pub enum NodeData {
    Element(ElementData),
    Text(RefCell<String>),
    Comment(RefCell<String>),
    Doctype(Doctype),
    Document(Cell<QuirksMode>),
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

/// A node inside a DOM-like tree.
pub struct Node<'a> {
    parent: Cell<Option<&'a Node<'a>>>,
    previous_sibling: Cell<Option<&'a Node<'a>>>,
    next_sibling: Cell<Option<&'a Node<'a>>>,
    first_child: Cell<Option<&'a Node<'a>>>,
    last_child: Cell<Option<&'a Node<'a>>>,
    pub data: NodeData,
}


fn same_ref<T>(a: &T, b: &T) -> bool {
    a as *const T == b as *const T
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
    ///
    /// Typically, this node needs to be moved into an arena allocator
    /// before it can be used in a tree.
    pub fn new(data: NodeData) -> Node<'a> {
        Node {
            parent: Cell::new(None),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
            previous_sibling: Cell::new(None),
            next_sibling: Cell::new(None),
            data: data,
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

    /// Returns whether two references point to the same node.
    pub fn same_node(&self, other: &Node<'a>) -> bool {
        same_ref(self, other)
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
    pub fn preceding_siblings(&'a self) -> PrecedingSiblings<'a> {
        PrecedingSiblings(Some(self))
    }

    /// Return an iterator of references to this node and the siblings after it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn following_siblings(&'a self) -> FollowingSiblings<'a> {
        FollowingSiblings(Some(self))
    }

    /// Return an iterator of references to this node’s children.
    pub fn children(&self) -> Children<'a> {
        Children(self.first_child.get())
    }

    /// Return an iterator of references to this node’s children, in reverse order.
    pub fn reverse_children(&self) -> ReverseChildren<'a> {
        ReverseChildren(self.last_child.get())
    }

    /// Return an iterator of references to this node and its descendants, in tree order.
    ///
    /// Parent nodes appear before the descendants.
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn descendants(&'a self) -> Descendants<'a> {
        Descendants(self.traverse())
    }

    /// Return an iterator of references to this node and its descendants, in tree order.
    pub fn traverse(&'a self) -> Traverse<'a> {
        Traverse {
            root: self,
            next: Some(NodeEdge::Start(self)),
        }
    }

    /// Return an iterator of references to this node and its descendants, in tree order.
    pub fn reverse_traverse(&'a self) -> ReverseTraverse<'a> {
        ReverseTraverse {
            root: self,
            next: Some(NodeEdge::End(self)),
        }
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
            debug_assert!(last_child.next_sibling.get().is_none());
            last_child.next_sibling.set(Some(new_child));
        } else {
            debug_assert!(self.first_child.get().is_none());
            self.first_child.set(Some(new_child));
        }
        self.last_child.set(Some(new_child));
    }

    /// Prepend a new child to this node, before existing children.
    pub fn prepend(&'a self, new_child: &'a Node<'a>) {
        new_child.detach();
        new_child.parent.set(Some(self));
        if let Some(first_child) = self.first_child.take() {
            debug_assert!(first_child.previous_sibling.get().is_none());
            first_child.previous_sibling.set(Some(new_child));
            new_child.next_sibling.set(Some(first_child));
        } else {
            debug_assert!(self.first_child.get().is_none());
            self.last_child.set(Some(new_child));
        }
        self.first_child.set(Some(new_child));
    }

    /// Insert a new sibling after this node.
    pub fn insert_after(&'a self, new_sibling: &'a Node<'a>) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent.get());
        new_sibling.previous_sibling.set(Some(self));
        if let Some(next_sibling) = self.next_sibling.take() {
            debug_assert!(same_ref(next_sibling.previous_sibling.get().unwrap(), self));
            next_sibling.previous_sibling.set(Some(new_sibling));
            new_sibling.next_sibling.set(Some(next_sibling));
        } else if let Some(parent) = self.parent.get() {
            debug_assert!(same_ref(parent.last_child.get().unwrap(), self));
            parent.last_child.set(Some(new_sibling));
        }
        self.next_sibling.set(Some(new_sibling));
    }

    /// Insert a new sibling before this node.
    pub fn insert_before(&'a self, new_sibling: &'a Node<'a>) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent.get());
        new_sibling.next_sibling.set(Some(self));
        if let Some(previous_sibling) = self.previous_sibling.take() {
            new_sibling.previous_sibling.set(Some(previous_sibling));
            debug_assert!(same_ref(previous_sibling.next_sibling.get().unwrap(), self));
            previous_sibling.next_sibling.set(Some(new_sibling));
        } else if let Some(parent) = self.parent.get() {
            debug_assert!(same_ref(parent.first_child.get().unwrap(), self));
            parent.first_child.set(Some(new_sibling));
        }
        self.previous_sibling.set(Some(new_sibling));
    }
}


macro_rules! axis_iterator {
    (#[$attr:meta] $name: ident: $next: ident) => {
        #[$attr]
        pub struct $name<'a>(Option<&'a Node<'a>>);

        impl<'a> Iterator for $name<'a> {
            type Item = &'a Node<'a>;

            fn next(&mut self) -> Option<&'a Node<'a>> {
                match self.0.take() {
                    Some(node) => {
                        self.0 = node.$next.get();
                        Some(node)
                    }
                    None => None
                }
            }
        }
    }
}

axis_iterator! {
    #[doc = "An iterator of references to the ancestors a given node."]
    Ancestors: parent
}

axis_iterator! {
    #[doc = "An iterator of references to the siblings before a given node."]
    PrecedingSiblings: previous_sibling
}

axis_iterator! {
    #[doc = "An iterator of references to the siblings after a given node."]
    FollowingSiblings: next_sibling
}

axis_iterator! {
    #[doc = "An iterator of references to the children of a given node."]
    Children: next_sibling
}

axis_iterator! {
    #[doc = "An iterator of references to the children of a given node, in reverse order."]
    ReverseChildren: previous_sibling
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


#[derive(Debug, Clone)]
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

macro_rules! traverse_iterator {
    (#[$attr:meta] $name: ident: $first_child: ident, $next_sibling: ident) => {
        #[$attr]
        pub struct $name<'a> {
            root: &'a Node<'a>,
            next: Option<NodeEdge<&'a Node<'a>>>,
        }

        impl<'a> Iterator for $name<'a> {
            type Item = NodeEdge<&'a Node<'a>>;

            fn next(&mut self) -> Option<NodeEdge<&'a Node<'a>>> {
                match self.next.take() {
                    Some(item) => {
                        self.next = match item {
                            NodeEdge::Start(node) => {
                                match node.$first_child.get() {
                                    Some(child) => Some(NodeEdge::Start(child)),
                                    None => Some(NodeEdge::End(node))
                                }
                            }
                            NodeEdge::End(node) => {
                                if node.same_node(self.root) {
                                    None
                                } else {
                                    match node.$next_sibling.get() {
                                        Some(sibling) => Some(NodeEdge::Start(sibling)),
                                        None => match node.parent.get() {
                                            Some(parent) => Some(NodeEdge::End(parent)),

                                            // `node.parent()` here can only be `None`
                                            // if the tree has been modified during iteration,
                                            // but silently stoping iteration
                                            // seems a more sensible behavior than panicking.
                                            None => None
                                        }
                                    }
                                }
                            }
                        };
                        Some(item)
                    }
                    None => None
                }
            }
        }
    }
}

traverse_iterator! {
    #[doc = "An iterator of the start and end edges of a given node and its descendants, in tree order."]
    Traverse: first_child, next_sibling
}

traverse_iterator! {
    #[doc = "An iterator of the start and end edges of a given node and its descendants, in reverse tree order."]
    ReverseTraverse: last_child, previous_sibling
}
