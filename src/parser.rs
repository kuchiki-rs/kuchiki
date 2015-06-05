use html5ever::{self, Attribute};
use html5ever::tree_builder::{TreeSink, NodeOrText, QuirksMode};
use std::borrow::Cow;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use string_cache::QualName;
use typed_arena::Arena;


use tree::Node;
use std::option;

pub struct Html<F = IgnoreParseErrors> where F: FnMut(Cow<'static, str>) {
    opts: ParseOpts<F>,
    data: option::IntoIter<String>,
}

impl Html  {
    pub fn from_string(string: &str) -> Html {
        Html {
            opts: ParseOpts::default(),
            data: Some(string.to_string()).into_iter(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Html {
        let mut buf = String::new();
        let mut file = File::open(&path).unwrap();
        file.read_to_string(&mut buf).unwrap();
        Html {
            opts: ParseOpts::default(),
            data: Some(buf).into_iter(),
        }
    }
}

impl<F> Html <F> where F: FnMut(Cow<'static, str>) {
    pub fn parse<'a>(self, arena: &'a Arena<Node<'a>>) -> &'a Node<'a> {
        let parser = Parser {
            arena : arena,
            document_node: Node::new_document(arena),
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


struct Parser<'a, F> where F: FnMut(Cow<'static, str>) {
    arena: &'a Arena<Node<'a>>,
    document_node: &'a Node<'a>,
    on_parse_error: F,
}


impl<'a, F> TreeSink for Parser<'a, F> where F: FnMut(Cow<'static, str>) {
    type Handle = &'a Node<'a>;

    fn parse_error(&mut self, message: Cow<'static, str>) {
        (self.on_parse_error)(message);
    }

    fn get_document(&mut self) -> &'a Node<'a> {
        self.document_node
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.document_node.as_document().unwrap()._quirks_mode.set(mode)
    }

    fn same_node(&self, x: &'a Node<'a>, y: &'a Node<'a>) -> bool {
        x == y
    }

    fn elem_name(&self, target: &'a Node<'a>) -> QualName {
        target.as_element().unwrap().name.clone()
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>) -> &'a Node<'a> {
        let attrs = attrs.into_iter().map(|Attribute { name, value }| (name, value));
        Node::new_element(name, attrs, self.arena)
    }

    fn create_comment(&mut self, text: String) -> &'a Node<'a> {
        Node::new_comment(text, self.arena)
    }

    fn append(&mut self, parent: &'a Node<'a>, child: NodeOrText<&'a Node<'a>>) {
        match child {
            NodeOrText::AppendNode(node) => parent.append(node),
            NodeOrText::AppendText(text) => {
                if let Some(last_child) = parent.last_child() {
                    if let Some(existing) = last_child.as_text() {
                        existing.borrow_mut().push_str(&text);
                        return
                    }
                }
                parent.append(Node::new_text(text, self.arena))
            }
        }
    }

    fn append_before_sibling(&mut self, sibling: &'a Node<'a>, child: NodeOrText<&'a Node<'a>>)
                             -> Result<(), NodeOrText<&'a Node<'a>>> {
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
                sibling.insert_before(Node::new_text(text, self.arena))
            }
        }
        Ok(())
    }

    fn append_doctype_to_document(&mut self, name: String, public_id: String, system_id: String) {
        self.document_node.append(Node::new_doctype(name, public_id, system_id, self.arena))
    }

    fn add_attrs_if_missing(&mut self, target: &'a Node<'a>, attrs: Vec<Attribute>) {
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

    fn remove_from_parent(&mut self, target: &'a Node<'a>) {
        target.detach()
    }

    fn reparent_children(&mut self, node: &'a Node<'a>, new_parent: &'a Node<'a>) {
        // FIXME: Can this be done more effciently in rctree,
        // by moving the whole linked list of children at once?
        for child in node.children() {
            new_parent.append(child)
        }
    }

    fn mark_script_already_started(&mut self, _node: &'a Node<'a>) {
        // FIXME: Is this useful outside of a browser?
    }
}
