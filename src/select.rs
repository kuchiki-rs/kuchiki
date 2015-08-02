use selectors::{self, parser, matching};
use selectors::parser::{AttrSelector, NamespaceConstraint, Selector};
use string_cache::{Atom, Namespace, QualName};

use tree::{NodeRef, NodeData, ElementData};
use iter::{NodeIterator, Select};
use node_data_ref::NodeDataRef;


impl selectors::Element for NodeDataRef<ElementData> {
    fn parent_element(&self) -> Option<Self> {
        self.as_node().parent().and_then(NodeRef::into_element_ref)
    }
    fn first_child_element(&self) -> Option<Self> {
        self.as_node().children().elements().next()
    }
    fn last_child_element(&self) -> Option<Self> {
        self.as_node().children().rev().elements().next()
    }
    fn prev_sibling_element(&self) -> Option<Self> {
        self.as_node().preceding_siblings().elements().next()
    }
    fn next_sibling_element(&self) -> Option<Self> {
        self.as_node().following_siblings().elements().next()
    }
    fn is_empty(&self) -> bool {
        self.as_node().children().all(|child| match child.data {
            NodeData::Element(_) => false,
            NodeData::Text(ref text) => text.borrow().is_empty(),
            _ => true,
        })
    }
    fn is_root(&self) -> bool {
        match self.as_node().parent() {
            None => false,
            Some(parent) => matches!(parent.data, NodeData::Document(_))
        }
    }

    fn is_html_element_in_html_document(&self) -> bool {
        // FIXME: Have a notion of HTML document v.s. XML document?
        self.name.ns == ns!(html)
    }
    fn get_local_name<'a>(&'a self) -> &'a Atom { &self.name.local }
    fn get_namespace<'a>(&'a self) -> &'a Namespace { &self.name.ns }
    fn get_hover_state(&self) -> bool { false }
    fn get_focus_state(&self) -> bool { false }
    fn get_id(&self) -> Option<Atom> {
        self.attributes.borrow().get(&QualName::new(ns!(""), atom!(id))).map(|s| Atom::from_slice(s))
    }
    fn get_disabled_state(&self) -> bool { false }
    fn get_enabled_state(&self) -> bool { false }
    fn get_checked_state(&self) -> bool { false }
    fn get_indeterminate_state(&self) -> bool { false }
    fn has_class(&self, name: &Atom) -> bool {
        !name.is_empty() &&
        if let Some(class_attr) = self.attributes.borrow().get(&QualName::new(ns!(""), atom!(class))) {
            class_attr.split(::selectors::matching::SELECTOR_WHITESPACE)
            .any(|class| name.as_slice() == class )
        } else {
            false
        }
    }
    fn is_link(&self) -> bool {
        self.name.ns == ns!(html) &&
        matches!(self.name.local, atom!(a) | atom!(area) | atom!(link)) &&
        self.attributes.borrow().contains_key(&QualName::new(ns!(""), atom!(href)))
    }
    fn is_visited_link(&self) -> bool { false }
    fn is_unvisited_link(&self) -> bool { self.is_link() }
    fn each_class<F>(&self, mut callback: F) where F: FnMut(&Atom) {
        if let Some(class_attr) = self.attributes.borrow().get(&QualName::new(ns!(""), atom!(class))) {
            for class in class_attr.split(::selectors::matching::SELECTOR_WHITESPACE) {
                if !class.is_empty() {
                    callback(&Atom::from_slice(class))
                }
            }
        }
    }
    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool where F: Fn(&str) -> bool {
        let name = if self.is_html_element_in_html_document() {
            &attr.lower_name
        } else {
            &attr.name
        };
        self.attributes.borrow().iter().any(|(key, value)| {
            !matches!(attr.namespace, NamespaceConstraint::Specific(ref ns) if *ns != key.ns) &&
            key.local == *name &&
            test(value)
        })
    }
}


pub struct Selectors(Vec<Selector>);

impl Selectors {
    pub fn compile(s: &str) -> Result<Selectors, ()> {
        parser::parse_author_origin_selector_list_from_str(s).map(Selectors)
    }

    pub fn matches(&self, element: &NodeDataRef<ElementData>) -> bool {
        matching::matches(&self.0, element, None)
    }

    pub fn filter<I>(&self, iter: I) -> Select<I, &Selectors>
    where I: Iterator<Item=NodeDataRef<ElementData>> {
        Select {
            iter: iter,
            selectors: self,
        }
    }
}

impl ::std::str::FromStr for Selectors {
    type Err = ();
    fn from_str(s: &str) -> Result<Selectors, ()> {
        Selectors::compile(s)
    }
}
