extern crate kuchiki;
extern crate html5ever;

use kuchiki::traits::*;

pub fn get_node_attribute(node: &kuchiki::NodeRef, attr: &str) -> Option<String> {
    node.as_element().and_then(|e| {
        e.attributes
            .borrow()
            .get(attr)
            .and_then(|v| Some(v.to_string()))
    })
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

    let id = get_node_attribute(&node, "id");

    // Some("container")
    println!("Id: {:?}", id);
}
