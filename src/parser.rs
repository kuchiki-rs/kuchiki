use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Error};
use std::option;
use std::path::Path;
use html5ever::{self, Attribute};
use html5ever::tree_builder::{TreeSink, NodeOrText, QuirksMode};
use string_cache::QualName;

use tree::NodeRef;

pub struct Html<F = IgnoreParseErrors> where F: FnMut(Cow<'static, str>) {
    opts: ParseOpts<F>,
    data: option::IntoIter<String>,
}

impl Html  {
    pub fn from_string<S: Into<String>>(string: S) -> Html {
        Html {
            opts: ParseOpts::default(),
            data: Some(string.into()).into_iter(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Html, Error> {
        let mut buf = String::new();
        let mut file = try!(File::open(&path));
        file.read_to_string(&mut buf).unwrap();
        Ok(Html {
            opts: ParseOpts::default(),
            data: Some(buf).into_iter(),
        })
    }
}

impl<F> Html <F> where F: FnMut(Cow<'static, str>) {
    pub fn parse(self) -> NodeRef {
        let parser = Parser {
            document_node: NodeRef::new_document(),
            on_parse_error: self.opts.on_parse_error,
        };
        let html5opts = html5ever::ParseOpts {
            tokenizer: self.opts.tokenizer,
            tree_builder: self.opts.tree_builder,
        };
        let parser = html5ever::parse_to(parser, self.data, html5opts);
        parser.document_node
    }
}

pub struct ParseOpts<F =IgnoreParseErrors> where F: FnMut(Cow<'static, str>) {
    pub tokenizer: html5ever::tokenizer::TokenizerOpts,
    pub tree_builder: html5ever::tree_builder::TreeBuilderOpts,
    pub on_parse_error: F,
}

pub struct IgnoreParseErrors;

impl<Args> FnOnce<Args> for IgnoreParseErrors {
    type Output = ();
    extern "rust-call" fn call_once(self, _args: Args) {}
}

impl<Args> FnMut<Args> for IgnoreParseErrors {
    extern "rust-call" fn call_mut(&mut self, _args: Args) {}
}

impl Default for ParseOpts<IgnoreParseErrors> {
    fn default() -> ParseOpts<IgnoreParseErrors> {
        ParseOpts {
            tokenizer: Default::default(),
            tree_builder: Default::default(),
            on_parse_error: IgnoreParseErrors,
        }
    }
}


struct Parser<F> where F: FnMut(Cow<'static, str>) {
    document_node: NodeRef,
    on_parse_error: F,
}


impl<F> TreeSink for Parser<F> where F: FnMut(Cow<'static, str>) {
    type Handle = NodeRef;

    fn parse_error(&mut self, message: Cow<'static, str>) {
        (self.on_parse_error)(message);
    }

    fn get_document(&mut self) -> NodeRef {
        self.document_node.clone()
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.document_node.as_document().unwrap()._quirks_mode.set(mode)
    }

    fn same_node(&self, x: NodeRef, y: NodeRef) -> bool {
        x == y
    }

    fn elem_name(&self, target: NodeRef) -> QualName {
        target.as_element().unwrap().name.clone()
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>) -> NodeRef {
        let attrs = attrs.into_iter().map(|Attribute { name, value }| (name, value));
        NodeRef::new_element(name, attrs)
    }

    fn create_comment(&mut self, text: String) -> NodeRef {
        NodeRef::new_comment(text)
    }

    fn append(&mut self, parent: NodeRef, child: NodeOrText<NodeRef>) {
        match child {
            NodeOrText::AppendNode(node) => parent.append(node),
            NodeOrText::AppendText(text) => {
                if let Some(last_child) = parent.last_child() {
                    if let Some(existing) = last_child.as_text() {
                        existing.borrow_mut().push_str(&text);
                        return
                    }
                }
                parent.append(NodeRef::new_text(text))
            }
        }
    }

    fn append_before_sibling(&mut self, sibling: NodeRef, child: NodeOrText<NodeRef>)
                             -> Result<(), NodeOrText<NodeRef>> {
        if sibling.parent().is_none() {
            return Err(child)
        }
        match child {
            NodeOrText::AppendNode(node) => sibling.insert_before(node),
            NodeOrText::AppendText(text) => {
                if let Some(previous_sibling) = sibling.previous_sibling() {
                    if let Some(existing) = previous_sibling.as_text() {
                        existing.borrow_mut().push_str(&text);
                        return Ok(())
                    }
                }
                sibling.insert_before(NodeRef::new_text(text))
            }
        }
        Ok(())
    }

    fn append_doctype_to_document(&mut self, name: String, public_id: String, system_id: String) {
        self.document_node.append(NodeRef::new_doctype(name, public_id, system_id))
    }

    fn add_attrs_if_missing(&mut self, target: NodeRef, attrs: Vec<Attribute>) {
        // FIXME: https://github.com/servo/html5ever/issues/121
        if let Some(element) = target.as_element() {
            let mut attributes = element.attributes.borrow_mut();
            for Attribute { name, value } in attrs {
                use std::collections::hash_map::Entry;
                match attributes.entry(name) {
                    Entry::Vacant(entry) => {
                        entry.insert(value);
                    }
                    Entry::Occupied(mut entry) => {
                        *entry.get_mut() = value;
                    }
                }
            }
        }
    }

    fn remove_from_parent(&mut self, target: NodeRef) {
        target.detach()
    }

    fn reparent_children(&mut self, node: NodeRef, new_parent: NodeRef) {
        // FIXME: Can this be done more effciently in rctree,
        // by moving the whole linked list of children at once?
        for child in node.children() {
            new_parent.append(child)
        }
    }

    fn mark_script_already_started(&mut self, _node: NodeRef) {
        // FIXME: Is this useful outside of a browser?
    }
}
