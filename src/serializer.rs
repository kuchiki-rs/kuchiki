use html5ever::serialize::{Serializable, Serializer, TraversalScope, serialize};
use html5ever::serialize::TraversalScope::*;
use std::io::{Write, Result};
use std::string::ToString;

use tree::{Node, NodeData};


impl<'a> Serializable for Node<'a> {
    fn serialize<'wr, Wr: Write>(&self, serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope) -> Result<()> {
        match (traversal_scope, &self.data) {
            (_, &NodeData::Element(ref element)) => {
                if traversal_scope == IncludeNode {
                    try!(serializer.start_elem(
                        element.name.clone(),
                        element.attributes.borrow().iter().map(|(name, value)| (name, &**value))));
                }

                for child in self.children() {
                    try!(child.serialize(serializer, IncludeNode));
                }

                if traversal_scope == IncludeNode {
                    try!(serializer.end_elem(element.name.clone()));
                }
                Ok(())
            }

            (ChildrenOnly, &NodeData::Document(_)) => {
                for child in self.children() {
                    try!(child.serialize(serializer, IncludeNode));
                }
                Ok(())
            }

            (ChildrenOnly, _) => Ok(()),

            (IncludeNode, &NodeData::Doctype(ref doctype)) => serializer.write_doctype(&doctype.name),
            (IncludeNode, &NodeData::Text(ref text)) => serializer.write_text(&text.borrow()),
            (IncludeNode, &NodeData::Comment(ref text)) => serializer.write_comment(&text.borrow()),

            (IncludeNode, &NodeData::Document(_)) => panic!("Can't serialize Document node itself"),
        }
    }
}


impl<'a> ToString for Node<'a> {
    fn to_string(&self) -> String {
        let mut utf_vec = Vec::new();
        let result = match serialize(&mut utf_vec, self, Default::default()) {
            Ok(_) => match String::from_utf8(utf_vec)  {
                Ok(s) => s,
                Err(_) => String::new(),
            },
            Err(_) => String::new(),
        };
        result
    }
}
