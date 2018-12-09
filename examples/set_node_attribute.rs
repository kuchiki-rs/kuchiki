extern crate kuchiki;
extern crate html5ever;

use kuchiki::traits::*;

pub fn set_node_attribute(node: &kuchiki::NodeRef, attribute: &str, value: &str) -> bool {
    if let Some(element) = node.as_element() {
        let attr = kuchiki::Attribute {
            prefix: None,
            value: value.to_string(),
        };
        *element
            .attributes
            .borrow_mut()
            .entry(attribute)
            .or_insert(attr.clone()) = attr.clone();
        return true;
    }

    false
}

fn main() {
    let html = r"
        <DOCTYPE html>
        <html>
        <head></head>
        <body> <div id='container'></div> </body>
        </html>
    ";

    let document = kuchiki::parse_html().one(html);
    let container = document.select("#container").unwrap().next().unwrap();
    let node = container.as_node().clone();

    // Div with id="container"
    println!("Before: {:?}", node);

    set_node_attribute(&node, "id", "container2");

    // Div with id="container2"
    println!("After: {:?}", node);
}
