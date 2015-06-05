use html5ever::tree_builder::QuirksMode;
use selectors::tree::TNode;
use typed_arena::Arena;
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
fn select() {
    let arena = Arena::new();
    let html = r"
<title>Test case</title>
<p class=foo>Foo
<p>Bar
";
    let document = Html::from_string(html).parse(&arena);
    let selectors = ::selectors::parser::parse_author_origin_selector_list_from_str("p.foo").unwrap();
    let matching = document.descendants()
    .filter(|node| node.is_element() && ::selectors::matching::matches(&selectors, node, &None))
    .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1);
    assert_eq!(&**matching[0].first_child().unwrap().as_text().unwrap().borrow(), "Foo\n");
}
