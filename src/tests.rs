use html5ever::tree_builder::QuirksMode;
use std::path::Path;

use tempdir::TempDir;
use super::{Html};

#[test]
fn text_iter() {
    let html = r"
<!doctype html>
<title>Test case</title>
<p>Content contains <b>Important</b> data</p>";
    let document = Html::from_string(html).parse();
    let paragraph = document.select("p").unwrap().collect::<Vec<_>>();
    assert_eq!(paragraph.len(), 1);
    let texts = paragraph[0].as_node().text_iter().collect::<Vec<_>>();
    assert_eq!(texts.len(), 3);
    assert_eq!(&*texts[0].borrow(), "Content contains ");
    assert_eq!(&*texts[1].borrow(), "Important");
    assert_eq!(&*texts[2].borrow(), " data");
    {
        let mut x = texts[0].borrow_mut();
        &x.truncate(0);
        &x.push_str("Content doesn't contain ");
    }
    assert_eq!(&*texts[0].borrow(), "Content doesn't contain ");
}

#[test]
fn parse_and_serialize() {
    let html = r"
<!doctype html>
<title>Test case</title>
<p>Content";
    let document = Html::from_string(html).parse();
    assert_eq!(document.as_document().unwrap().quirks_mode(), QuirksMode::NoQuirks);
    assert_eq!(document.to_string(), r"<!DOCTYPE html>
<html><head><title>Test case</title>
</head><body><p>Content</p></body></html>");
}

#[test]
fn parse_file() {
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
    let document = Html::from_file(&path).unwrap().parse();
    assert_eq!(document.to_string(), html);
}

#[test]
fn serialize_and_read_file() {
    let tempdir = TempDir::new("test_rm_tempdir").unwrap();
    let mut path = tempdir.path().to_path_buf();
    path.push("temp.html");

    let html = r"<!DOCTYPE html><html><head><title>Title</title></head><body>Body</body></html>";
    let document = Html::from_string(html).parse();
    let _ = document.serialize_to_file(path.clone());

    let document2 = Html::from_file(&path).unwrap().parse();
    assert_eq!(document.to_string(), document2.to_string());
}

#[test]
fn select() {
    let html = r"
<title>Test case</title>
<p class=foo>Foo
<p>Bar
<p class=foo>Foo
";

    let document = Html::from_string(html).parse();
    let matching = document.select("p.foo").unwrap().collect::<Vec<_>>();
    assert_eq!(matching.len(), 2);
    let child = matching[0].as_node().first_child().unwrap();
    assert_eq!(&**child.as_text().unwrap().borrow(), "Foo\n");
}

#[test]
fn to_string() {
    let html = r"<!DOCTYPE html>
<html>
    <head>
        <title>Test case</title>
    </head>
    <body>
        <p class=foo>Foo
    </body>
</html>";

    let document = Html::from_string(html).parse();
    assert_eq!(document.descendants().nth(11).unwrap().to_string(), "<p class=\"foo\">Foo\n    \n</p>");
}
