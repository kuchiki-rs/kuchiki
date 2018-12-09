extern crate kuchiki;
extern crate html5ever;

use std::collections::HashMap;
use kuchiki::traits::*;
use html5ever::{LocalName, Namespace, QualName};

pub fn create_new_element(tag_name: &str) -> kuchiki::NodeRef {
    let attributes: HashMap<kuchiki::ExpandedName, kuchiki::Attribute> = HashMap::default();
    let name = QualName::new(None, Namespace::from(""), LocalName::from(tag_name));
    kuchiki::NodeRef::new_element(name, attributes)
}

pub fn rename_node(node: &kuchiki::NodeRef, new_tag_name: &str) -> kuchiki::NodeRef {
    let new_node = create_new_element(new_tag_name);
    if let Some(element) = node.as_element() {
        let old_attributes = element.attributes.borrow().clone();
        *new_node.as_element().unwrap().attributes.borrow_mut() = old_attributes;
    }

    for child in node.children() {
        child.detach();
        new_node.append(child);
    }
    node.insert_after(new_node.clone());
    node.detach();

    new_node
}

fn main() {
    let html = r"
        <DOCTYPE html>
        <html>
        <head></head>
        <body>
            <div id='container'>
                <h1>Example</h1>
                <p class='foo'>paragraph</p>
            </div>
        </body>
        </html>
    ";

    let document = kuchiki::parse_html().one(html);
    let container = document.select("#container").unwrap().next().unwrap();
    let mut node = container.as_node().clone();

    // div
    println!("Before: {:?}", node);

    node = rename_node(&node, "article");

    // article
    println!("After: {:?}", node);
}
