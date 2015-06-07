use html5ever::tree_builder::QuirksMode;
use selectors::tree::TNode;
use typed_arena::Arena;
use std::path::Path;

use super::{Html};

#[test]
fn parse_and_serialize() {
    let arena = Arena::new();
    let html = r"
<!doctype html>
<title>Test case</title>
<p>Content";
    let document = Html::from_string(html).parse(&arena);
    assert_eq!(document.as_document().unwrap().quirks_mode(), QuirksMode::NoQuirks);
    assert_eq!(document.to_string(), r"<!DOCTYPE html>
<html><head><title>Test case</title>
</head><body><p>Content</p></body></html>");
}

#[test]
fn parse_file() {
    let arena = Arena::new();
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    path.push("test_data".to_string());
    path.push("foo.html");

    let html = r"<!DOCTYPE html>
<html><head>
        <title>Test case</title>
    </head>
    <body>
        <p>Foo</p>
    

</body></html>";
    let document = Html::from_file(&path).unwrap().parse(&arena);
    assert_eq!(document.to_string(), html);
}

#[test]
fn select() {
    let arena = Arena::new();
    let html = r"
<title>Test case</title>
<p class=foo>Foo
<p>Bar
<p class=foo>Foo
";

    let document = Html::from_string(html).parse(&arena);
    let matching = document.select("p.foo").unwrap().collect::<Vec<_>>();
    assert_eq!(matching.len(), 2);
    assert_eq!(&**matching[0].first_child().unwrap().as_text().unwrap().borrow(), "Foo\n");
}
