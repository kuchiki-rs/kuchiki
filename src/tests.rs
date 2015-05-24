use html5ever::serialize::serialize;
use html5ever::tree_builder::QuirksMode;
use tree::NodeData;
use typed_arena::Arena;


#[test]
fn it_works() {
    let arena = Arena::new();
    let html = r"
<!doctype html>
<title>Test case</title>
<p>Content";
    let document = ::parse(Some(html.into()), &arena, Default::default());
    assert!(if let &NodeData::Document(QuirksMode::NoQuirks) = &*document.data.borrow() {
        true
    } else {
        false
    });
    let mut serialized = Vec::new();
    serialize(&mut serialized, document, Default::default()).unwrap();
    assert_eq!(String::from_utf8(serialized).unwrap(), r"<!DOCTYPE html
<html><head><title>Test case</title>
</head><body><p>Content</p></body></html>");
}
