use gc::Gc;
use move_cell::MoveCell;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use html5ever::tree_builder::QuirksMode;
use string_cache::QualName;

use iter::NodeIterator;


/// Node data specific to the node type.
#[derive(Debug, PartialEq, Clone)]
pub enum NodeData {
    /// Element node
    Element(ElementData),

    /// Text node
    Text(RefCell<String>),

    /// Comment node
    Comment(RefCell<String>),

    /// Doctype node
    Doctype(Doctype),

    /// Document node
    Document(DocumentData),
}

/// Data specific to doctype nodes.
#[derive(Debug, PartialEq, Clone)]
pub struct Doctype {
    /// The name of the doctype
    pub name: String,

    /// The public ID of the doctype
    pub public_id: String,

    /// The system ID of the doctype
    pub system_id: String,
}

/// Data specific to element nodes.
#[derive(Debug, PartialEq, Clone)]
pub struct ElementData {
    /// The namespace and local name of the element, such as `ns!(html)` and `body`.
    pub name: QualName,

    /// The attributes of the elements.
    pub attributes: RefCell<HashMap<QualName, String>>,
}

/// Data specific to document nodes.
#[derive(Debug, PartialEq, Clone)]
pub struct DocumentData {
    #[doc(hidden)]
    pub _quirks_mode: Cell<QuirksMode>,
}

impl DocumentData {
    /// The quirks mode of the document, as determined by the HTML parser.
    #[inline]
    pub fn quirks_mode(&self) -> QuirksMode {
        self._quirks_mode.get()
    }
}

/// A garbage-collected reference to a node.
#[derive(Clone)]
pub struct NodeRef(pub Gc<Node>);

impl fmt::Debug for NodeRef {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&*self.0, f)
    }
}

impl Deref for NodeRef {
    type Target = Node;
    #[inline]
    fn deref(&self) -> &Node {
        &*self.0
    }
}

impl Eq for NodeRef {}
impl PartialEq for NodeRef {
    #[inline]
    fn eq(&self, other: &NodeRef) -> bool {
        let a: *const Node = &*self.0;
        let b: *const Node = &*other.0;
        a == b
    }
}

/// A node inside a DOM-like tree.
#[derive(Trace)]
pub struct Node {
    parent: MoveCell<Option<Gc<Node>>>,
    previous_sibling: MoveCell<Option<Gc<Node>>>,
    next_sibling: MoveCell<Option<Gc<Node>>>,
    first_child: MoveCell<Option<Gc<Node>>>,
    last_child: MoveCell<Option<Gc<Node>>>,
    #[unsafe_ignore_trace] data: NodeData,
}

impl fmt::Debug for Node {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?} @ {:?}", self.data, self as *const Node)
    }
}

impl NodeRef {
    /// Create a new node.
    #[inline]
    pub fn new(data: NodeData) -> NodeRef {
        NodeRef(Gc::new(Node {
            parent: MoveCell::new(None),
            first_child: MoveCell::new(None),
            last_child: MoveCell::new(None),
            previous_sibling: MoveCell::new(None),
            next_sibling: MoveCell::new(None),
            data: data,
        }))
    }

    /// Create a new element node.
    #[inline]
    pub fn new_element<I>(name: QualName, attributes: I) -> NodeRef
                          where I: IntoIterator<Item=(QualName, String)> {
        NodeRef::new(NodeData::Element(ElementData {
            name: name,
            attributes: RefCell::new(attributes.into_iter().collect()),
        }))
    }

    /// Create a new text node.
    #[inline]
    pub fn new_text<T: Into<String>>(value: T) -> NodeRef {
        NodeRef::new(NodeData::Text(RefCell::new(value.into())))
    }

    /// Create a new comment node.
    #[inline]
    pub fn new_comment<T: Into<String>>(value: T) -> NodeRef {
        NodeRef::new(NodeData::Comment(RefCell::new(value.into())))
    }

    /// Create a new doctype node.
    #[inline]
    pub fn new_doctype<T1, T2, T3>(name: T1, public_id: T2, system_id: T3) -> NodeRef
                                   where T1: Into<String>, T2: Into<String>, T3: Into<String> {
        NodeRef::new(NodeData::Doctype(Doctype {
            name: name.into(),
            public_id: public_id.into(),
            system_id: system_id.into(),
        }))
    }

    /// Create a new document node.
    #[inline]
    pub fn new_document() -> NodeRef {
        NodeRef::new(NodeData::Document(DocumentData {
            _quirks_mode: Cell::new(QuirksMode::NoQuirks),
        }))
    }

    /// Return the concatenation of all text nodes in this subtree.
    pub fn text_contents(&self) -> String {
        let mut s = String::new();
        for text_node in self.inclusive_descendants().text_nodes() {
            s.push_str(&text_node.borrow());
        }
        s
    }
}

impl Node {
    /// Return a reference to this node’s node-type-specific data.
    #[inline]
    pub fn data(&self) -> &NodeData {
        &self.data
    }

    /// If this node is an element, return a reference to element-specific data.
    #[inline]
    pub fn as_element(&self) -> Option<&ElementData> {
        match self.data {
            NodeData::Element(ref value) => Some(value),
            _ => None
        }
    }

    /// If this node is a text node, return a reference to its contents.
    #[inline]
    pub fn as_text(&self) -> Option<&RefCell<String>> {
        match self.data {
            NodeData::Text(ref value) => Some(value),
            _ => None
        }
    }

    /// If this node is a comment, return a reference to its contents.
    #[inline]
    pub fn as_comment(&self) -> Option<&RefCell<String>> {
        match self.data {
            NodeData::Comment(ref value) => Some(value),
            _ => None
        }
    }

    /// If this node is a document, return a reference to doctype-specific data.
    #[inline]
    pub fn as_doctype(&self) -> Option<&Doctype> {
        match self.data {
            NodeData::Doctype(ref value) => Some(value),
            _ => None
        }
    }

    /// If this node is a document, return a reference to document-specific data.
    #[inline]
    pub fn as_document(&self) -> Option<&DocumentData> {
        match self.data {
            NodeData::Document(ref value) => Some(value),
            _ => None
        }
    }

    /// Return a reference to the parent node, unless this node is the root of the tree.
    #[inline]
    pub fn parent(&self) -> Option<NodeRef> {
        self.parent.clone_inner().map(NodeRef)
    }

    /// Return a reference to the first child of this node, unless it has no child.
    #[inline]
    pub fn first_child(&self) -> Option<NodeRef> {
        self.first_child.clone_inner().map(NodeRef)
    }

    /// Return a reference to the last child of this node, unless it has no child.
    #[inline]
    pub fn last_child(&self) -> Option<NodeRef> {
        self.last_child.clone_inner().map(NodeRef)
    }

    /// Return a reference to the previous sibling of this node, unless it is a first child.
    #[inline]
    pub fn previous_sibling(&self) -> Option<NodeRef> {
        self.previous_sibling.clone_inner().map(NodeRef)
    }

    /// Return a reference to the previous sibling of this node, unless it is a last child.
    #[inline]
    pub fn next_sibling(&self) -> Option<NodeRef> {
        self.next_sibling.clone_inner().map(NodeRef)
    }

    /// Detach a node from its parent and siblings. Children are not affected.
    ///
    /// To remove a node and its descendants, detach it and drop any strong reference to it.
    pub fn detach(&self) {
        let parent_opt = self.parent.take();
        let previous_sibling_opt = self.previous_sibling.take();
        let next_sibling_opt = self.next_sibling.take();

        if let Some(ref next_sibling) = next_sibling_opt {
            next_sibling.previous_sibling.set(previous_sibling_opt.clone());
        } else if let Some(ref parent) = parent_opt {
            parent.last_child.set(previous_sibling_opt.clone());
        }

        if let Some(previous_sibling) = previous_sibling_opt {
            previous_sibling.next_sibling.set(next_sibling_opt);
        } else if let Some(parent) = parent_opt {
            parent.first_child.set(next_sibling_opt);
        }
    }
}

impl NodeRef {
    /// Append a new child to this node, after existing children.
    ///
    /// The new child is detached from its previous position.
    pub fn append(&self, new_child: NodeRef) {
        new_child.detach();
        new_child.parent.set(Some(self.0.clone()));
        if let Some(last_child) = self.last_child.replace(Some(new_child.0.clone())) {
            debug_assert!(last_child.next_sibling.is_none());
            last_child.next_sibling.set(Some(new_child.0.clone()));
            new_child.previous_sibling.set(Some(last_child));
        } else {
            debug_assert!(self.first_child.is_none());
            self.first_child.set(Some(new_child.0));
        }
    }

    /// Prepend a new child to this node, before existing children.
    ///
    /// The new child is detached from its previous position.
    pub fn prepend(&self, new_child: NodeRef) {
        new_child.detach();
        new_child.parent.set(Some(self.0.clone()));
        if let Some(first_child) = self.first_child.take() {
            debug_assert!(first_child.previous_sibling.is_none());
            first_child.previous_sibling.set(Some(new_child.0.clone()));
            new_child.next_sibling.set(Some(first_child));
        } else {
            debug_assert!(self.first_child.is_none());
            self.last_child.set(Some(new_child.0.clone()));
        }
        self.first_child.set(Some(new_child.0));
    }

    /// Insert a new sibling after this node.
    ///
    /// The new sibling is detached from its previous position.
    pub fn insert_after(&self, new_sibling: NodeRef) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent.clone_inner());
        new_sibling.previous_sibling.set(Some(self.0.clone()));
        if let Some(next_sibling) = self.next_sibling.take() {
            debug_assert!(next_sibling.previous_sibling().unwrap() == *self);
            next_sibling.previous_sibling.set(Some(new_sibling.0.clone()));
            new_sibling.next_sibling.set(Some(next_sibling));
        } else if let Some(parent) = self.parent() {
            debug_assert!(parent.last_child().unwrap() == *self);
            parent.last_child.set(Some(new_sibling.0.clone()));
        }
        self.next_sibling.set(Some(new_sibling.0));
    }

    /// Insert a new sibling before this node.
    ///
    /// The new sibling is detached from its previous position.
    pub fn insert_before(&self, new_sibling: NodeRef) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent.clone_inner());
        new_sibling.next_sibling.set(Some(self.0.clone()));
        if let Some(previous_sibling) = self.previous_sibling.replace(Some(new_sibling.0.clone())) {
            debug_assert!(previous_sibling.next_sibling().unwrap() == *self);
            previous_sibling.next_sibling.set(Some(new_sibling.0.clone()));
            new_sibling.previous_sibling.set(Some(previous_sibling));
        } else if let Some(parent) = self.parent() {
            debug_assert!(parent.first_child().unwrap() == *self);
            parent.first_child.set(Some(new_sibling.0));
        }
    }
}
