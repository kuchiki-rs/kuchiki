extern crate kuchiki;
extern crate html5ever;

use kuchiki::traits::*;
use html5ever::serialize::{TraversalScope, SerializeOpts};

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
    let node = container.as_node();

    let mut u8_vec = Vec::new();
    node.serialize_with(&mut u8_vec, SerializeOpts {
        traversal_scope: TraversalScope::ChildrenOnly(None),
        ..Default::default()
    }).unwrap();

    let inner_html = String::from_utf8(u8_vec).unwrap();

    // <h1>Example</h1>
    // <p class="foo">paragraph</p>
    println!("HTML: {}", inner_html);
}
