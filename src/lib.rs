extern crate rctree;
extern crate html5ever;
extern crate string_cache;

pub use html5ever::tree_builder::QuirksMode;
pub use rctree::{Ref, RefMut, Ancestors, PrecedingSiblings, FollowingSiblings,
                 Children, ReverseChildren, Descendants, Traverse, ReverseTraverse, NodeEdge};
pub use string_cache::{Atom, Namespace, QualName};

use html5ever::Attribute;
use html5ever::tree_builder::{TreeSink, NodeOrText};
use std::borrow::Cow;
use std::collections::HashMap;


pub type NodeRef = rctree::NodeRef<NodeData>;

pub struct Tree {
    pub document_node: NodeRef,
    pub errors: Vec<Cow<'static, str>>,
    pub quirks_mode: QuirksMode,
}


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
    name: QualName,
    attributes: HashMap<QualName, String>,
}


impl TreeSink for Tree {
    type Handle = NodeRef;

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        self.errors.push(msg);
    }

    fn get_document(&mut self) -> NodeRef {
        self.document_node.clone()
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.quirks_mode = mode;
    }

    fn same_node(&self, x: NodeRef, y: NodeRef) -> bool {
        x.same_node(&y)
    }

    fn elem_name(&self, target: NodeRef) -> QualName {
        let borrow = target.borrow();
        match *borrow {
            NodeData::Element(ref element) => element.name.clone(),
            _ => panic!("not an element!"),
        }
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>) -> NodeRef {
        NodeRef::new(NodeData::Element(ElementData {
            name: name,
            attributes: attrs.into_iter().map(|Attribute { name, value }| (name, value)).collect()
        }))
    }

    fn create_comment(&mut self, text: String) -> NodeRef {
        NodeRef::new(NodeData::Comment(text))
    }

    fn append(&mut self, parent: NodeRef, child: NodeOrText<NodeRef>) {
        match child {
            NodeOrText::AppendNode(node) => parent.append(node),
            NodeOrText::AppendText(text) => {
                if let Some(last_child) = parent.last_child() {
                    let mut borrow = last_child.borrow_mut();
                    if let &mut NodeData::Text(ref mut existing) = &mut *borrow {
                        existing.push_str(&text);
                        return
                    }
                }
                parent.append(NodeRef::new(NodeData::Text(text)))
            }
        }
    }

    fn append_before_sibling(&mut self, sibling: NodeRef, child: NodeOrText<NodeRef>)
                             -> Result<(), NodeOrText<NodeRef>> {
        if sibling.parent().is_none() {
            return Err(child)
        }
        match child {
            NodeOrText::AppendNode(node) => sibling.insert_before(node),
            NodeOrText::AppendText(text) => {
                if let Some(previous_sibling) = sibling.previous_sibling() {
                    let mut borrow = previous_sibling.borrow_mut();
                    if let &mut NodeData::Text(ref mut existing) = &mut *borrow {
                        existing.push_str(&text);
                        return Ok(())
                    }
                }
                sibling.insert_before(NodeRef::new(NodeData::Text(text)))
            }
        }
        Ok(())
    }

    fn append_doctype_to_document(&mut self, name: String, public_id: String, system_id: String) {
        self.document_node.append(NodeRef::new(NodeData::Doctype(Doctype {
            name: name,
            public_id: public_id,
            system_id: system_id,
        })))
    }

    fn add_attrs_if_missing(&mut self, target: NodeRef, attrs: Vec<Attribute>) {
        let mut borrow = target.borrow_mut();
        // FIXME: https://github.com/servo/html5ever/issues/121
        if let &mut NodeData::Element(ref mut element) = &mut *borrow {
            for Attribute { name, value } in attrs {
                use std::collections::hash_map::Entry;
                match element.attributes.entry(name) {
                    Entry::Vacant(entry) => {
                        entry.insert(value);
                    }
                    Entry::Occupied(mut entry) => {
                        *entry.get_mut() = value;
                    }
                }
            }
        }
    }

    fn remove_from_parent(&mut self, target: NodeRef) {
        target.detach()
    }

    fn reparent_children(&mut self, node: NodeRef, new_parent: NodeRef) {
        // FIXME: Can this be done more effciently in rctree,
        // by moving the whole linked list of children at once?
        for child in node.children() {
            new_parent.append(child)
        }
    }

    fn mark_script_already_started(&mut self, _node: NodeRef) {
        // FIXME: Is this useful outside of a browser?
    }
}
