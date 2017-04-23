use cssparser::ToCss;
use iter::{NodeIterator, Select};
use node_data_ref::NodeDataRef;
use ref_slice::ref_slice;
use selectors::{self, parser, matching, Element};
use selectors::parser::{AttrSelector, NamespaceConstraint, SelectorImpl, ParserContext};
use selectors::parser::Selector as GenericSelector;
use std::ascii::AsciiExt;
use std::fmt;
use html5ever::{LocalName, Namespace};
use tree::{NodeRef, NodeData, ElementData};

/// The definition of whitespace per CSS Selectors Level 3 § 4.
///
/// Copied from rust-selectors.
static SELECTOR_WHITESPACE: &'static [char] = &[' ', '\t', '\n', '\r', '\x0C'];

#[derive(Debug)]
pub struct KuchikiSelectors;

impl SelectorImpl for KuchikiSelectors {
    type AttrValue = String;
    type Identifier = LocalName;
    type ClassName = LocalName;
    type LocalName = LocalName;
    type NamespacePrefix = LocalName;
    type NamespaceUrl = Namespace;
    type BorrowedNamespaceUrl = Namespace;
    type BorrowedLocalName = LocalName;

    type NonTSPseudoClass = PseudoClass;
    fn parse_non_ts_pseudo_class(_context: &ParserContext<Self>, name: &str) -> Result<PseudoClass, ()> {
        use self::PseudoClass::*;
             if name.eq_ignore_ascii_case("any-link") { Ok(AnyLink) }
        else if name.eq_ignore_ascii_case("link") { Ok(Link) }
        else if name.eq_ignore_ascii_case("visited") { Ok(Visited) }
        else if name.eq_ignore_ascii_case("active") { Ok(Active) }
        else if name.eq_ignore_ascii_case("focus") { Ok(Focus) }
        else if name.eq_ignore_ascii_case("hover") { Ok(Hover) }
        else if name.eq_ignore_ascii_case("enabled") { Ok(Enabled) }
        else if name.eq_ignore_ascii_case("disabled") { Ok(Disabled) }
        else if name.eq_ignore_ascii_case("checked") { Ok(Checked) }
        else if name.eq_ignore_ascii_case("indeterminate") { Ok(Indeterminate) }
        else { Err(()) }
    }

    type PseudoElement = PseudoElement;
    fn parse_pseudo_element(_context: &ParserContext<Self>, _name: &str) -> Result<PseudoElement, ()> {
        Err(())
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum PseudoClass {
    AnyLink,
    Link,
    Visited,
    Active,
    Focus,
    Hover,
    Enabled,
    Disabled,
    Checked,
    Indeterminate,
}

impl ToCss for PseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str(match *self {
            PseudoClass::AnyLink => ":any-link",
            PseudoClass::Link => ":link",
            PseudoClass::Visited => ":visited",
            PseudoClass::Active => ":active",
            PseudoClass::Focus => ":focus",
            PseudoClass::Hover => ":hover",
            PseudoClass::Enabled => ":enabled",
            PseudoClass::Disabled => ":disabled",
            PseudoClass::Checked => ":checked",
            PseudoClass::Indeterminate => ":indeterminate",
        })
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum PseudoElement {}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
        }
    }
}

impl selectors::Element for NodeDataRef<ElementData> {
    #[inline]
    fn parent_element(&self) -> Option<Self> {
        self.as_node().parent().and_then(NodeRef::into_element_ref)
    }
    #[inline]
    fn first_child_element(&self) -> Option<Self> {
        self.as_node().children().elements().next()
    }
    #[inline]
    fn last_child_element(&self) -> Option<Self> {
        self.as_node().children().rev().elements().next()
    }
    #[inline]
    fn prev_sibling_element(&self) -> Option<Self> {
        self.as_node().preceding_siblings().elements().next()
    }
    #[inline]
    fn next_sibling_element(&self) -> Option<Self> {
        self.as_node().following_siblings().elements().next()
    }
    #[inline]
    fn is_empty(&self) -> bool {
        self.as_node().children().all(|child| match *child.data() {
            NodeData::Element(_) => false,
            NodeData::Text(ref text) => text.borrow().is_empty(),
            _ => true,
        })
    }
    #[inline]
    fn is_root(&self) -> bool {
        match self.as_node().parent() {
            None => false,
            Some(parent) => matches!(*parent.data(), NodeData::Document(_))
        }
    }

    #[inline]
    fn is_html_element_in_html_document(&self) -> bool {
        // FIXME: Have a notion of HTML document v.s. XML document?
        self.name.ns == ns!(html)
    }
    #[inline] fn get_local_name<'a>(&'a self) -> &'a LocalName { &self.name.local }
    #[inline] fn get_namespace<'a>(&'a self) -> &'a Namespace { &self.name.ns }
    #[inline]
    fn get_id(&self) -> Option<LocalName> {
        self.attributes.borrow().get(local_name!("id")).map(LocalName::from)
    }
    #[inline]
    fn has_class(&self, name: &LocalName) -> bool {
        !name.is_empty() &&
        if let Some(class_attr) = self.attributes.borrow().get(local_name!("class")) {
            class_attr.split(SELECTOR_WHITESPACE)
            .any(|class| &**name == class )
        } else {
            false
        }
    }
    #[inline]
    fn each_class<F>(&self, mut callback: F) where F: FnMut(&LocalName) {
        if let Some(class_attr) = self.attributes.borrow().get(local_name!("class")) {
            for class in class_attr.split(SELECTOR_WHITESPACE) {
                if !class.is_empty() {
                    callback(&LocalName::from(class))
                }
            }
        }
    }

    fn match_non_ts_pseudo_class(&self, pseudo: PseudoClass) -> bool {
        use self::PseudoClass::*;
        match pseudo {
            Active | Focus | Hover | Enabled | Disabled | Checked | Indeterminate | Visited => false,
            AnyLink | Link => {
                self.name.ns == ns!(html) &&
                matches!(self.name.local, local_name!("a") | local_name!("area") | local_name!("link")) &&
                self.attributes.borrow().contains(local_name!("href"))
            }
        }
    }
}

impl selectors::MatchAttrGeneric for NodeDataRef<ElementData> {
    type Impl = KuchikiSelectors;

    #[inline]
    fn match_attr<F>(&self, attr: &AttrSelector<Self::Impl>, test: F) -> bool where F: Fn(&str) -> bool {
        let name = if self.is_html_element_in_html_document() {
            &attr.lower_name
        } else {
            &attr.name
        };
        self.attributes.borrow().map.iter().any(|(key, value)| {
            !matches!(attr.namespace, NamespaceConstraint::Specific(ref ns) if ns.url != key.ns) &&
            key.local == *name &&
            test(value)
        })
    }
}


/// A pre-compiled list of CSS Selectors.
pub struct Selectors(pub Vec<Selector>);

/// A pre-compiled CSS Selector.
pub struct Selector(GenericSelector<KuchikiSelectors>);

/// The specificity of a selector.
///
/// Opaque, but ordered.
///
/// Determines precedence in the cascading algorithm.
/// When equal, a rule later in source order takes precedence.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Specificity(u32);

impl Selectors {
    /// Compile a list of selectors. This may fail on syntax errors or unsupported selectors.
    #[inline]
    pub fn compile(s: &str) -> Result<Selectors, ()> {
        parser::parse_author_origin_selector_list_from_str(s).map(|vec| {
            Selectors(vec.into_iter().map(Selector).collect())
        })
    }

    /// Returns whether the given element matches this list of selectors.
    #[inline]
    pub fn matches(&self, element: &NodeDataRef<ElementData>) -> bool {
        self.0.iter().any(|s| s.matches(element))
    }

    /// Filter an element iterator, yielding those matching this list of selectors.
    #[inline]
    pub fn filter<I>(&self, iter: I) -> Select<I, &Selectors>
    where I: Iterator<Item=NodeDataRef<ElementData>> {
        Select {
            iter: iter,
            selectors: self,
        }
    }
}

impl Selector {
    /// Returns whether the given element matches this selector.
    #[inline]
    pub fn matches(&self, element: &NodeDataRef<ElementData>) -> bool {
        matching::matches(ref_slice(&self.0), element, None, matching::MatchingReason::Other)
    }

    /// Return the specificity of this selector.
    pub fn specificity(&self) -> Specificity {
        Specificity(self.0.specificity)
    }
}

impl ::std::str::FromStr for Selectors {
    type Err = ();
    #[inline]
    fn from_str(s: &str) -> Result<Selectors, ()> {
        Selectors::compile(s)
    }
}
