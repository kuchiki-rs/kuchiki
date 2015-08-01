use move_cell::MoveCell;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use html5ever::tree_builder::QuirksMode;
use rc::{Rc, Weak};
use string_cache::QualName;


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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NodeRef(pub Rc<Node>);

impl Deref for NodeRef {
    type Target = Node;
    fn deref(&self) -> &Node { &*self.0 }
}

/// A node inside a DOM-like tree.
pub struct Node {
    parent: MoveCell<Option<Weak<Node>>>,
    previous_sibling: MoveCell<Option<Weak<Node>>>,
    next_sibling: MoveCell<Option<Rc<Node>>>,
    first_child: MoveCell<Option<Rc<Node>>>,
    last_child: MoveCell<Option<Weak<Node>>>,
    pub data: NodeData,
}

impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self as *const Node == other as *const Node
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?} @ {:?}", self.data, self as *const Node)
    }
}

/// Prevent implicit recursion when dropping nodes to avoid overflowing the stack.
///
/// The implicit drop is correct, but recursive.
/// In the worst case (where no node has both a next sibling and a child),
/// a tree of a few tens of thousands of nodes could cause a stack overflow.
///
/// This `Drop` implementations makes sure the recursion does not happen.
/// Instead, it has an explicit `Vec<Rc<Node>>` stack to traverse the subtree,
/// but only following `Rc<Node>` references that are "unique":
/// that have a strong reference count of 1.
/// Those are the nodes that would have been dropped recursively.
///
/// The stack holds ancestors of the current node rather than preceding siblings,
/// on the assumption that large document trees are typically wider than deep.
impl Drop for Node {
    fn drop(&mut self) {
        // `.take_if_unique_strong()` temporarily leaves the tree in an inconsistent state,
        // as the corresponding `Weak` reference in the other direction is not removed.
        // It is important that all `Some(_)` strong references it returns
        // are dropped by the end of this `drop` call,
        // and that no user code is invoked in-between.
        let mut stack = Vec::new();
        if let Some(rc) = self.first_child.take_if_unique_strong() {
            non_recursive_drop_unique_rc(rc, &mut stack);
        }
        if let Some(rc) = self.next_sibling.take_if_unique_strong() {
            non_recursive_drop_unique_rc(rc, &mut stack);
        }

        fn non_recursive_drop_unique_rc(mut rc: Rc<Node>, stack: &mut Vec<Rc<Node>>) {
            loop {
                if let Some(child) = rc.first_child.take_if_unique_strong() {
                    stack.push(rc);
                    rc = child;
                    continue
                }
                if let Some(sibling) = rc.next_sibling.take_if_unique_strong() {
                    // The previous  value of `rc: Rc<Node>` is dropped here.
                    // Since it was unique, the corresponding `Node` is dropped as well.
                    // `<Node as Drop>::drop` does not call `drop_rc`
                    // as both the first child and next sibling were already taken.
                    // Weak reference counts decremented here for `MoveCell`s that are `Some`:
                    // * `rc.parent`: still has a strong reference in `stack` or elsewhere
                    // * `rc.last_child`: this is the last weak ref. Deallocated now.
                    // * `rc.previous_sibling`: this is the last weak ref. Deallocated now.
                    rc = sibling;
                    continue
                }
                if let Some(parent) = stack.pop() {
                    // Same as in the above comment.
                    rc = parent;
                    continue
                }
                return
            }
        }
    }
}

impl NodeRef {
    /// Create a new node from its associated data.
    pub fn new(data: NodeData) -> NodeRef {
        NodeRef(Rc::new(Node {
            parent: MoveCell::new(None),
            first_child: MoveCell::new(None),
            last_child: MoveCell::new(None),
            previous_sibling: MoveCell::new(None),
            next_sibling: MoveCell::new(None),
            data: data,
        }))
    }

    pub fn new_element<I>(name: QualName, attributes: I) -> NodeRef
                          where I: IntoIterator<Item=(QualName, String)> {
        NodeRef::new(NodeData::Element(ElementData {
            name: name,
            attributes: RefCell::new(attributes.into_iter().collect()),
        }))
    }

    pub fn new_text<T: Into<String>>(value: T) -> NodeRef {
        NodeRef::new(NodeData::Text(RefCell::new(value.into())))
    }

    pub fn new_comment<T: Into<String>>(value: T) -> NodeRef {
        NodeRef::new(NodeData::Comment(RefCell::new(value.into())))
    }

    pub fn new_doctype<T1, T2, T3>(name: T1, public_id: T2, system_id: T3) -> NodeRef
                                   where T1: Into<String>, T2: Into<String>, T3: Into<String> {
        NodeRef::new(NodeData::Doctype(Doctype {
            name: name.into(),
            public_id: public_id.into(),
            system_id: system_id.into(),
        }))
    }

    pub fn new_document() -> NodeRef {
        NodeRef::new(NodeData::Document(DocumentData {
            _quirks_mode: Cell::new(QuirksMode::NoQuirks),
        }))
    }
}

impl Node {
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
    pub fn parent(&self) -> Option<NodeRef> {
        self.parent.upgrade().map(NodeRef)
    }

    /// Return a reference to the first child of this node, unless it has no child.
    pub fn first_child(&self) -> Option<NodeRef> {
        self.first_child.clone_inner().map(NodeRef)
    }

    /// Return a reference to the last child of this node, unless it has no child.
    pub fn last_child(&self) -> Option<NodeRef> {
        self.last_child.upgrade().map(NodeRef)
    }

    /// Return a reference to the previous sibling of this node, unless it is a first child.
    pub fn previous_sibling(&self) -> Option<NodeRef> {
        self.previous_sibling.upgrade().map(NodeRef)
    }

    /// Return a reference to the previous sibling of this node, unless it is a last child.
    pub fn next_sibling(&self) -> Option<NodeRef> {
        self.next_sibling.clone_inner().map(NodeRef)
    }
}

impl NodeRef {
    pub fn into_element_ref(self) -> Option<NodeDataRef<ElementData>> {
        NodeDataRef::new_opt(self, Node::as_element)
    }

    pub fn into_text_ref(self) -> Option<NodeDataRef<RefCell<String>>> {
        NodeDataRef::new_opt(self, Node::as_text)
    }

    pub fn into_comment_ref(self) -> Option<NodeDataRef<RefCell<String>>> {
        NodeDataRef::new_opt(self, Node::as_comment)
    }

    pub fn into_doctype_ref(self) -> Option<NodeDataRef<Doctype>> {
        NodeDataRef::new_opt(self, Node::as_doctype)
    }

    pub fn into_document_ref(self) -> Option<NodeDataRef<DocumentData>> {
        NodeDataRef::new_opt(self, Node::as_document)
    }
}

impl Node {
    /// Detach a node from its parent and siblings. Children are not affected.
    pub fn detach(&self) {
        let parent_weak = self.parent.take();
        let previous_sibling_weak = self.previous_sibling.take();
        let next_sibling_strong = self.next_sibling.take();

        let previous_sibling_opt = previous_sibling_weak.as_ref().and_then(|weak| weak.upgrade());

        if let Some(next_sibling_ref) = next_sibling_strong.as_ref() {
            next_sibling_ref.previous_sibling.set(previous_sibling_weak);
        } else if let Some(parent_ref) = parent_weak.as_ref() {
            if let Some(parent_strong) = parent_ref.upgrade() {
                parent_strong.last_child.set(previous_sibling_weak);
            }
        }

        if let Some(previous_sibling_strong) = previous_sibling_opt {
            previous_sibling_strong.next_sibling.set(next_sibling_strong);
        } else if let Some(parent_ref) = parent_weak.as_ref() {
            if let Some(parent_strong) = parent_ref.upgrade() {
                parent_strong.first_child.set(next_sibling_strong);
            }
        }
    }
}

impl NodeRef {
    /// Append a new child to this node, after existing children.
    pub fn append(&self, new_child: NodeRef) {
        new_child.detach();
        new_child.parent.set(Some(self.0.downgrade()));
        if let Some(last_child_weak) = self.last_child.replace(Some(new_child.0.downgrade())) {
            if let Some(last_child) = last_child_weak.upgrade() {
                new_child.previous_sibling.set(Some(last_child_weak));
                debug_assert!(last_child.next_sibling.is_none());
                last_child.next_sibling.set(Some(new_child.0));
                return
            }
        }
        debug_assert!(self.first_child.is_none());
        self.first_child.set(Some(new_child.0));
    }

    /// Prepend a new child to this node, before existing children.
    pub fn prepend(&self, new_child: NodeRef) {
        new_child.detach();
        new_child.parent.set(Some(self.0.downgrade()));
        if let Some(first_child) = self.first_child.take() {
            debug_assert!(first_child.previous_sibling.is_none());
            first_child.previous_sibling.set(Some(new_child.0.downgrade()));
            new_child.next_sibling.set(Some(first_child));
        } else {
            debug_assert!(self.first_child.is_none());
            self.last_child.set(Some(new_child.0.downgrade()));
        }
        self.first_child.set(Some(new_child.0));
    }

    /// Insert a new sibling after this node.
    pub fn insert_after(&self, new_sibling: NodeRef) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent.clone_inner());
        new_sibling.previous_sibling.set(Some(self.0.downgrade()));
        if let Some(next_sibling) = self.next_sibling.take() {
            debug_assert!(next_sibling.previous_sibling().unwrap() == *self);
            next_sibling.previous_sibling.set(Some(new_sibling.0.downgrade()));
            new_sibling.next_sibling.set(Some(next_sibling));
        } else if let Some(parent) = self.parent() {
            debug_assert!(parent.last_child().unwrap() == *self);
            parent.last_child.set(Some(new_sibling.0.downgrade()));
        }
        self.next_sibling.set(Some(new_sibling.0));
    }

    /// Insert a new sibling before this node.
    pub fn insert_before(&self, new_sibling: NodeRef) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent.clone_inner());
        new_sibling.next_sibling.set(Some(self.0.clone()));
        if let Some(previous_sibling_weak) = self.previous_sibling.replace(
                Some(new_sibling.0.downgrade())) {
            if let Some(previous_sibling) = previous_sibling_weak.upgrade() {
                new_sibling.previous_sibling.set(Some(previous_sibling_weak));
                debug_assert!(previous_sibling.next_sibling().unwrap() == *self);
                previous_sibling.next_sibling.set(Some(new_sibling.0));
                return
            }
        }
        if let Some(parent) = self.parent() {
            debug_assert!(parent.first_child().unwrap() == *self);
            parent.first_child.set(Some(new_sibling.0));
        }
    }
}


/// Holds a strong reference to a node, but derefs to some component inside of it.
pub struct NodeDataRef<T> {
    _keep_alive: NodeRef,
    _reference: *const T
}

impl<T> NodeDataRef<T> {
    /// Create a `NodeDataRef` for a component in a given node.
    pub fn new<F>(rc: NodeRef, f: F) -> NodeDataRef<T> where F: FnOnce(&Node) -> &T {
        NodeDataRef {
            _reference: f(&*rc),
            _keep_alive: rc,
        }
    }

    /// Create a `NodeDataRef` for and a component that may or may not be in a given node.
    pub fn new_opt<F>(rc: NodeRef, f: F) -> Option<NodeDataRef<T>>
        where F: FnOnce(&Node) -> Option<&T> {
        f(&*rc).map(|r| r as *const T).map(move |r| NodeDataRef {
            _reference: r,
            _keep_alive: rc,
        })
    }

    pub fn as_node(&self) -> &NodeRef {
        &self._keep_alive
    }
}

impl<T> Deref for NodeDataRef<T> {
    type Target = T;
    fn deref(&self) -> &T { unsafe { &*self._reference } }
}

impl<T: fmt::Debug> fmt::Debug for NodeDataRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&**self, f)
    }
}
