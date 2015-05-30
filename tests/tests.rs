extern crate html5ever;
extern crate selectors;
extern crate typed_arena;
extern crate kuchiki;

use html5ever::serialize::serialize;
use html5ever::tree_builder::QuirksMode;
use selectors::tree::TNode;
use typed_arena::Arena;


#[test]
fn parse_and_serialize() {
    let arena = Arena::new();
    let html = r"
<!doctype html>
<title>Test case</title>
<p>Content";
    let document = kuchiki::parse(Some(html.into()), &arena, Default::default());
    assert_eq!(document.as_document().unwrap().quirks_mode(), QuirksMode::NoQuirks);
    let mut serialized = Vec::new();
    serialize(&mut serialized, document, Default::default()).unwrap();
    assert_eq!(String::from_utf8(serialized).unwrap(), r"<!DOCTYPE html>
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
    let document = kuchiki::parse(Some(html.into()), &arena, Default::default());
    let selectors = ::selectors::parser::parse_author_origin_selector_list_from_str("p.foo").unwrap();
    let matching = document.descendants()
    .filter(|node| node.is_element() && ::selectors::matching::matches(&selectors, node, &None))
    .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1);
    assert_eq!(&**matching[0].first_child().unwrap().as_text().unwrap().borrow(), "Foo\n");
}
