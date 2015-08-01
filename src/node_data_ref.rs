use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;
use tree::{Node, NodeRef, ElementData, Doctype, DocumentData};

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
