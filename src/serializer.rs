pub use html5ever::serialize::serialize;

use html5ever::serialize::{Serializable, Serializer, TraversalScope};
use html5ever::serialize::TraversalScope::*;
use std::io::{Write, Result};

use tree::{Node, NodeData};


impl<'a> Serializable for Node<'a> {
    fn serialize<'wr, Wr: Write>(&self, serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope) -> Result<()> {
        let node = self.data.borrow();
        match (traversal_scope, &*node) {
            (_, &NodeData::Element(ref element)) => {
                if traversal_scope == IncludeNode {
                    try!(serializer.start_elem(
                        element.name.clone(),
                        element.attributes.iter().map(|(name, value)| (name, &**value))));
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
            (IncludeNode, &NodeData::Text(ref text)) => serializer.write_text(&text),
            (IncludeNode, &NodeData::Comment(ref text)) => serializer.write_comment(&text),

            (IncludeNode, &NodeData::Document(_)) => panic!("Can't serialize Document node itself"),
        }
    }
}
