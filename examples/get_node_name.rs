extern crate kuchiki;
extern crate html5ever;

use kuchiki::traits::*;

pub fn get_node_name(node: &kuchiki::NodeRef) -> Option<String> {
    node.as_element().and_then(|e| Some(e.name.local.to_lowercase()))
}

fn main() {
    let html = r"
        <DOCTYPE html>
        <html>
        <head></head>
        <body><div id='container'></div></body>
        </html>
    ";

    let document = kuchiki::parse_html().one(html);
    let container = document.select("#container").unwrap().next().unwrap();
    let node = container.as_node();

    // Some("div")
    println!("Node name: {:?}", get_node_name(&node));
}
