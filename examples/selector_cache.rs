extern crate kuchiki;

use std::{collections::HashMap, borrow::Cow, cell::RefCell, str};
use kuchiki::traits::*;

pub struct DomHelper<'a> {
    selector_cache: RefCell<HashMap<Cow<'a, str>, kuchiki::Selectors>>,
}

impl<'a> DomHelper<'a> {
    pub fn new() -> Self {
        DomHelper {
            selector_cache: RefCell::new(HashMap::default()),
        }
    }

    #[inline(always)]
    pub fn select<S>(&self, node: &kuchiki::NodeRef, selector: S) -> Vec<kuchiki::NodeRef>
    where
        S: Into<Cow<'a, str>>,
    {
        let selector_raw = selector.into();
        let mut cache = self.selector_cache.borrow_mut();
        let selectors = cache.entry(selector_raw.clone()).or_insert(
            kuchiki::Selectors::compile(&selector_raw)
                .expect(&format!("Wrong selector: {}", &selector_raw)),
        );

        selectors
            .filter(node.inclusive_descendants().elements())
            .map(|e| e.as_node().clone())
            .collect::<Vec<_>>()
    }
}

fn main() {
    let html = r"
        <DOCTYPE html>
        <html>
        <head></head>
        <body><div id='container'><a href='#' data-test='sample'>link</a></div></body>
        </html>
    ";

    let document = kuchiki::parse_html().one(html);

    let dom = DomHelper::new();
    let selector = "body :first-child a[data-test='sample']";
    // Now this complex selector is parsed and cached
    // Further calls will not spend time on parsing
    let result = dom.select(&document, selector);

    // Vector with one NodeRef 'a' element
    println!("Result:\n {:?}", result);
}