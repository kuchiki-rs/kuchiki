extern crate kuchiki;
extern crate html5ever;

use kuchiki::traits::*;

pub fn remove_node_attribute(node: &kuchiki::NodeRef, attribute: &str) {
    if let Some(element) = node.as_element() {
        element.attributes.borrow_mut().remove(attribute);
    }
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

    remove_node_attribute(&node, "id");

    // Div without id attribute
    println!("After: {:?}", node);
}
