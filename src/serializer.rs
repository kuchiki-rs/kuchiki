use std::io::{Write, Result};
use std::string::ToString;
use html5ever::serialize::{Serializable, Serializer, TraversalScope, serialize, SerializeOpts};
use html5ever::serialize::TraversalScope::*;

use tree::{NodeRef, NodeData};


impl Serializable for NodeRef {
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
                    try!(Serializable::serialize(&child, serializer, IncludeNode));
                }

                if traversal_scope == IncludeNode {
                    try!(serializer.end_elem(element.name.clone()));
                }
                Ok(())
            }

            (ChildrenOnly, &NodeData::Document(_)) => {
                for child in self.children() {
                    try!(Serializable::serialize(&child, serializer, IncludeNode));
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


impl ToString for NodeRef {
    fn to_string(&self) -> String {
        let mut u8_vec = Vec::new();
        self.serialize(&mut u8_vec);
        String::from_utf8(u8_vec).unwrap()
    }
}

impl NodeRef {
    fn serialize<W: Write>(&self, writer: &mut W) {
        let traversal_scope = match self.data {
            NodeData::Document(_) => ChildrenOnly,
            _ => IncludeNode,
        };

        serialize(writer, self, SerializeOpts {
            traversal_scope: traversal_scope,
            ..Default::default()
        }).unwrap();
    }
}
