extern crate kuchiki;
extern crate html5ever;

use std::collections::HashMap;
use html5ever::{LocalName, Namespace, QualName};

type Attributes = HashMap<kuchiki::ExpandedName, kuchiki::Attribute>;

fn create_attribute(name: &str, value: &str) -> (kuchiki::ExpandedName, kuchiki::Attribute) {
    (
        kuchiki::ExpandedName::new("", name),
        kuchiki::Attribute {
            prefix: None,
            value: value.to_string()
        }
    )
}

fn create_new_element(tag_name: &str, attributes: Attributes) -> kuchiki::NodeRef {
    let name = QualName::new(None, Namespace::from(""), LocalName::from(tag_name));
    kuchiki::NodeRef::new_element(name, attributes)
}

fn main() {
    let mut attributes = Attributes::default();

    let attribute = create_attribute("id", "test");
    attributes.insert(attribute.0, attribute.1);

    let new = create_new_element("div", attributes);

    // NodeRef(Element(ElementData { name: QualName { prefix: None, ns: Atom('' type=static), local: Atom('div' type=static) }, attributes: RefCell { value: Attributes { map: {ExpandedName { ns: Atom('' type=static), local: Atom('id' type=static) }: Attribute { prefix:None, value: "test" }} } }, template_contents: None }) )
    // In other words: <div id="test"></div>
    println!("New node: {:?}", new);
}
