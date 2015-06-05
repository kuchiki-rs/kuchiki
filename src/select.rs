use std::marker::PhantomData;

use selectors::parser;
use selectors::matching;
use selectors::tree::{TNode, TElement};
use selectors::parser::{AttrSelector, NamespaceConstraint, Selector};
use string_cache::{Atom, Namespace, QualName};
use tree::{Node, NodeData, ElementData, Descendants};


impl<'a> TNode<'a> for &'a Node<'a> {
    type Element = &'a ElementData;

    fn parent_node(self) -> Option<Self> { self.parent() }
    fn first_child(self) -> Option<Self> { self.first_child() }
    fn last_child(self) -> Option<Self> { self.last_child() }
    fn prev_sibling(self) -> Option<Self> { self.previous_sibling() }
    fn next_sibling(self) -> Option<Self> { self.next_sibling() }
    fn is_document(self) -> bool { matches!(self.data, NodeData::Document(_)) }
    fn is_element(self) -> bool { matches!(self.data, NodeData::Element(_)) }
    fn as_element(self) -> &'a ElementData { self.as_element().unwrap() }
    fn match_attr<F>(self, attr: &AttrSelector, test: F) -> bool where F: Fn(&str) -> bool {
        let name = if self.is_html_element_in_html_document() {
            &attr.lower_name
        } else {
            &attr.name
        };
        if let Some(element) = self.as_element() {
            element.attributes.borrow().iter().any(|(key, value)| {
                !matches!(attr.namespace, NamespaceConstraint::Specific(ref ns) if *ns != key.ns) &&
                key.local == *name &&
                test(value)
            })
        } else {
            false
        }
    }
    fn is_html_element_in_html_document(self) -> bool {
        matches!(self.data, NodeData::Element(ref element) if element.name.ns == ns!(html))
    }

    fn has_changed(self) -> bool { unimplemented!() }
    unsafe fn set_changed(self, _value: bool) { unimplemented!() }

    fn is_dirty(self) -> bool { unimplemented!() }
    unsafe fn set_dirty(self, _value: bool) { unimplemented!() }

    fn has_dirty_siblings(self) -> bool { unimplemented!() }
    unsafe fn set_dirty_siblings(self, _value: bool) { unimplemented!() }

    fn has_dirty_descendants(self) -> bool { unimplemented!() }
    unsafe fn set_dirty_descendants(self, _value: bool) { unimplemented!() }
}


impl<'a> TElement<'a> for &'a ElementData {
    fn get_local_name(self) -> &'a Atom { &self.name.local }
    fn get_namespace(self) -> &'a Namespace { &self.name.ns }
    fn get_hover_state(self) -> bool { false }
    fn get_focus_state(self) -> bool { false }
    fn get_id(self) -> Option<Atom> {
        self.attributes.borrow().get(&QualName::new(ns!(""), atom!(id))).map(|s| Atom::from_slice(s))
    }
    fn get_disabled_state(self) -> bool { false }
    fn get_enabled_state(self) -> bool { false }
    fn get_checked_state(self) -> bool { false }
    fn get_indeterminate_state(self) -> bool { false }
    fn has_class(self, name: &Atom) -> bool {
        !name.is_empty() &&
        if let Some(class_attr) = self.attributes.borrow().get(&QualName::new(ns!(""), atom!(class))) {
            class_attr.split(::selectors::matching::SELECTOR_WHITESPACE)
            .any(|class| name.as_slice() == class )
        } else {
            false
        }
    }
    fn has_nonzero_border(self) -> bool { false }
    fn is_link(self) -> bool {
        self.name.ns == ns!(html) &&
        matches!(self.name.local, atom!(a) | atom!(area) | atom!(link)) &&
        self.attributes.borrow().contains_key(&QualName::new(ns!(""), atom!(href)))
    }
    fn is_visited_link(self) -> bool { false }
    fn is_unvisited_link(self) -> bool { self.is_link() }
    fn each_class<F>(self, mut callback: F) where F: FnMut(&Atom) {
        if let Some(class_attr) = self.attributes.borrow().get(&QualName::new(ns!(""), atom!(class))) {
            for class in class_attr.split(::selectors::matching::SELECTOR_WHITESPACE) {
                if !class.is_empty() {
                    callback(&Atom::from_slice(class))
                }
            }
        }
    }
}

impl<'a> Node<'a> {
    pub fn css(&'a self, css_str: &str) -> FilterNodes<'a, Descendants<'a>> {
        let selectors = match parser::parse_author_origin_selector_list_from_str(css_str) {
            Ok(vec) => vec,
            Err(_) => Vec::new(),
        };
        FilterNodes{
            iter: self.descendants(),
            phantom: PhantomData,
            filter: selectors,
        }
    }
}

pub struct FilterNodes<'a, T:'a>
    where T: Iterator<Item=&'a Node<'a>>{
    iter: T,
    phantom: PhantomData<&'a T>,
    filter: Vec<Selector>,
}

impl<'a,T> Iterator for FilterNodes<'a,T> where T: Iterator<Item=&'a Node<'a>> {
    type Item = &'a Node<'a>;

    #[inline]
    fn next(&mut self) -> Option<&'a Node<'a>> {
        let mut next_node = None;
        for node in self.iter.by_ref() {
            if node.is_element() && matching::matches(&self.filter, &node, &None) {
                next_node = Some(node)
            }
        }
        next_node
    }
}

