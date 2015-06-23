use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::iter::Rev;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use html5ever::tree_builder::QuirksMode;
use movecell::MoveCell;
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
        self.parent.and_then(|weak| weak.upgrade()).map(NodeRef)
    }

    /// Return a reference to the first child of this node, unless it has no child.
    pub fn first_child(&self) -> Option<NodeRef> {
        self.first_child.clone_inner().map(NodeRef)
    }

    /// Return a reference to the last child of this node, unless it has no child.
    pub fn last_child(&self) -> Option<NodeRef> {
        self.last_child.and_then(|weak| weak.upgrade()).map(NodeRef)
    }

    /// Return a reference to the previous sibling of this node, unless it is a first child.
    pub fn previous_sibling(&self) -> Option<NodeRef> {
        self.previous_sibling.and_then(|weak| weak.upgrade()).map(NodeRef)
    }

    /// Return a reference to the previous sibling of this node, unless it is a last child.
    pub fn next_sibling(&self) -> Option<NodeRef> {
        self.next_sibling.clone_inner().map(NodeRef)
    }
}

impl NodeRef {
    pub fn into_element_ref(self) -> Option<NodeDataRef<ElementData>> {
        NodeDataRef::new_opt(self.0, Node::as_element)
    }

    pub fn into_text_ref(self) -> Option<NodeDataRef<RefCell<String>>> {
        NodeDataRef::new_opt(self.0, Node::as_text)
    }

    pub fn into_comment_ref(self) -> Option<NodeDataRef<RefCell<String>>> {
        NodeDataRef::new_opt(self.0, Node::as_comment)
    }

    pub fn into_doctype_ref(self) -> Option<NodeDataRef<Doctype>> {
        NodeDataRef::new_opt(self.0, Node::as_doctype)
    }

    pub fn into_document_ref(self) -> Option<NodeDataRef<DocumentData>> {
        NodeDataRef::new_opt(self.0, Node::as_document)
    }

    /// Return an iterator of references to this node and its ancestors.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn ancestors(&self) -> Ancestors {
        Ancestors(Some(self.clone()))
    }

    /// Return an iterator of references to this node and the siblings before it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn preceding_siblings(&self) -> Rev<Siblings> {
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

    /// Return an iterator of references to this node and the siblings after it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn following_siblings(&self) -> Siblings {
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
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    ///
    /// Note: this is the `NodeEdge::Start` items from `traverse()`.
    pub fn descendants(&self) -> Descendants {
        Descendants(self.traverse())
    }

    /// Return an iterator of the start and end edges of this node and its descendants,
    /// in tree order.
    pub fn traverse(&self) -> Traverse {
        Traverse(Some(State {
            next: NodeEdge::Start(self.clone()),
            next_back: NodeEdge::End(self.clone())
        }))
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
            next_sibling_ref.previous_sibling.replace(previous_sibling_weak);
        } else if let Some(parent_ref) = parent_weak.as_ref() {
            if let Some(parent_strong) = parent_ref.upgrade() {
                parent_strong.last_child.replace(previous_sibling_weak);
            }
        }

        if let Some(previous_sibling_strong) = previous_sibling_opt {
            previous_sibling_strong.next_sibling.replace(next_sibling_strong);
        } else if let Some(parent_ref) = parent_weak.as_ref() {
            if let Some(parent_strong) = parent_ref.upgrade() {
                parent_strong.first_child.replace(next_sibling_strong);
            }
        }
    }
}

impl NodeRef {
    /// Append a new child to this node, after existing children.
    pub fn append(&self, new_child: NodeRef) {
        new_child.detach();
        new_child.parent.replace(Some(self.0.downgrade()));
        if let Some(last_child_weak) = self.last_child.replace(Some(new_child.0.downgrade())) {
            if let Some(last_child) = last_child_weak.upgrade() {
                new_child.previous_sibling.replace(Some(last_child_weak));
                debug_assert!(last_child.next_sibling.is_none());
                last_child.next_sibling.replace(Some(new_child.0));
                return
            }
        }
        debug_assert!(self.first_child.is_none());
        self.first_child.replace(Some(new_child.0));
    }

    /// Prepend a new child to this node, before existing children.
    pub fn prepend(&self, new_child: NodeRef) {
        new_child.detach();
        new_child.parent.replace(Some(self.0.downgrade()));
        if let Some(first_child) = self.first_child.take() {
            debug_assert!(first_child.previous_sibling.is_none());
            first_child.previous_sibling.replace(Some(new_child.0.downgrade()));
            new_child.next_sibling.replace(Some(first_child));
        } else {
            debug_assert!(self.first_child.is_none());
            self.last_child.replace(Some(new_child.0.downgrade()));
        }
        self.first_child.replace(Some(new_child.0));
    }

    /// Insert a new sibling after this node.
    pub fn insert_after(&self, new_sibling: NodeRef) {
        new_sibling.detach();
        new_sibling.parent.replace(self.parent.clone_inner());
        new_sibling.previous_sibling.replace(Some(self.0.downgrade()));
        if let Some(next_sibling) = self.next_sibling.take() {
            debug_assert!(next_sibling.previous_sibling().unwrap() == *self);
            next_sibling.previous_sibling.replace(Some(new_sibling.0.downgrade()));
            new_sibling.next_sibling.replace(Some(next_sibling));
        } else if let Some(parent) = self.parent() {
            debug_assert!(parent.last_child().unwrap() == *self);
            parent.last_child.replace(Some(new_sibling.0.downgrade()));
        }
        self.next_sibling.replace(Some(new_sibling.0));
    }

    /// Insert a new sibling before this node.
    pub fn insert_before(&self, new_sibling: NodeRef) {
        new_sibling.detach();
        new_sibling.parent.replace(self.parent.clone_inner());
        new_sibling.next_sibling.replace(Some(self.0.clone()));
        if let Some(previous_sibling_weak) = self.previous_sibling.replace(
                Some(new_sibling.0.downgrade())) {
            if let Some(previous_sibling) = previous_sibling_weak.upgrade() {
                new_sibling.previous_sibling.replace(Some(previous_sibling_weak));
                debug_assert!(previous_sibling.next_sibling().unwrap() == *self);
                previous_sibling.next_sibling.replace(Some(new_sibling.0));
                return
            }
        }
        if let Some(parent) = self.parent() {
            debug_assert!(parent.first_child().unwrap() == *self);
            parent.first_child.replace(Some(new_sibling.0));
        }
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


/// Holds a strong reference to a node, but derefs to some component inside of it.
pub struct NodeDataRef<T> {
    _keep_alive: Rc<Node>,
    _reference: *const T
}

impl<T> NodeDataRef<T> {
    /// Create a `NodeDataRef` for a component in a given node.
    pub fn new<F>(rc: Rc<Node>, f: F) -> NodeDataRef<T> where F: FnOnce(&Node) -> &T {
        NodeDataRef {
            _reference: f(&*rc),
            _keep_alive: rc,
        }
    }

    /// Create a `NodeDataRef` for and a component that may or may not be in a given node.
    pub fn new_opt<F>(rc: Rc<Node>, f: F) -> Option<NodeDataRef<T>> where F: FnOnce(&Node) -> Option<&T> {
        f(&*rc).map(|r| r as *const T).map(move |r| NodeDataRef {
            _reference: r,
            _keep_alive: rc,
        })
    }
}

impl<T> Deref for NodeDataRef<T> {
    type Target = T;
    fn deref(&self) -> &T { unsafe { &*self._reference } }
}
