pub use html5ever::tree_builder::QuirksMode;

use html5ever::{self, Attribute};
use html5ever::tree_builder::{TreeSink, NodeOrText};
use std::borrow::Cow;
use string_cache::QualName;

use tree::{Arena, NodeRef, NodeData, Doctype, ElementData};


pub fn parse<'a, F, I>(source: I, arena: &'a Arena<'a>, opts: ParseOpts<F>) 
                       -> (NodeRef<'a>, QuirksMode)
                       where I: Iterator<Item=String>, F: FnMut(Cow<'static, str>) {
    let parser = Parser {
        arena: arena,
        document_node: arena.new_node(NodeData::Document),
        quirks_mode: QuirksMode::NoQuirks,
        on_parse_error: opts.on_parse_error,
    };
    let opts = html5ever::ParseOpts {
        tokenizer: opts.tokenizer,
        tree_builder: opts.tree_builder,
    };
    let parser = html5ever::parse_to(parser, source, opts);
    (parser.document_node, parser.quirks_mode)
}


pub struct ParseOpts<F> where F: FnMut(Cow<'static, str>) {
    pub tokenizer: html5ever::tokenizer::TokenizerOpts,
    pub tree_builder: html5ever::tree_builder::TreeBuilderOpts,
    pub on_parse_error: F,
}


struct Parser<'a, F> where F: FnMut(Cow<'static, str>) {
    arena: &'a Arena<'a>,
    document_node: NodeRef<'a>,
    quirks_mode: QuirksMode,
    on_parse_error: F,
}


impl<'a, F> TreeSink for Parser<'a, F> where F: FnMut(Cow<'static, str>) {
    type Handle = NodeRef<'a>;

    fn parse_error(&mut self, message: Cow<'static, str>) {
        (self.on_parse_error)(message);
    }

    fn get_document(&mut self) -> NodeRef<'a> {
        self.document_node
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.quirks_mode = mode;
    }

    fn same_node(&self, x: NodeRef<'a>, y: NodeRef<'a>) -> bool {
        x.same_node(y)
    }

    fn elem_name(&self, target: NodeRef<'a>) -> QualName {
        let borrow = target.borrow();
        match *borrow {
            NodeData::Element(ref element) => element.name.clone(),
            _ => panic!("not an element!"),
        }
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>) -> NodeRef<'a> {
        self.arena.new_node(NodeData::Element(ElementData {
            name: name,
            attributes: attrs.into_iter().map(|Attribute { name, value }| (name, value)).collect()
        }))
    }

    fn create_comment(&mut self, text: String) -> NodeRef<'a> {
        self.arena.new_node(NodeData::Comment(text))
    }

    fn append(&mut self, parent: NodeRef<'a>, child: NodeOrText<NodeRef<'a>>) {
        match child {
            NodeOrText::AppendNode(node) => parent.append(node),
            NodeOrText::AppendText(text) => {
                if let Some(last_child) = parent.last_child() {
                    let mut borrow = last_child.borrow_mut();
                    if let &mut NodeData::Text(ref mut existing) = &mut *borrow {
                        existing.push_str(&text);
                        return
                    }
                }
                parent.append(self.arena.new_node(NodeData::Text(text)))
            }
        }
    }

    fn append_before_sibling(&mut self, sibling: NodeRef<'a>, child: NodeOrText<NodeRef<'a>>)
                             -> Result<(), NodeOrText<NodeRef<'a>>> {
        if sibling.parent().is_none() {
            return Err(child)
        }
        match child {
            NodeOrText::AppendNode(node) => sibling.insert_before(node),
            NodeOrText::AppendText(text) => {
                if let Some(previous_sibling) = sibling.previous_sibling() {
                    let mut borrow = previous_sibling.borrow_mut();
                    if let &mut NodeData::Text(ref mut existing) = &mut *borrow {
                        existing.push_str(&text);
                        return Ok(())
                    }
                }
                sibling.insert_before(self.arena.new_node(NodeData::Text(text)))
            }
        }
        Ok(())
    }

    fn append_doctype_to_document(&mut self, name: String, public_id: String, system_id: String) {
        self.document_node.append(self.arena.new_node(NodeData::Doctype(Doctype {
            name: name,
            public_id: public_id,
            system_id: system_id,
        })))
    }

    fn add_attrs_if_missing(&mut self, target: NodeRef<'a>, attrs: Vec<Attribute>) {
        let mut borrow = target.borrow_mut();
        // FIXME: https://github.com/servo/html5ever/issues/121
        if let &mut NodeData::Element(ref mut element) = &mut *borrow {
            for Attribute { name, value } in attrs {
                use std::collections::hash_map::Entry;
                match element.attributes.entry(name) {
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

    fn remove_from_parent(&mut self, target: NodeRef<'a>) {
        target.detach()
    }

    fn reparent_children(&mut self, node: NodeRef<'a>, new_parent: NodeRef<'a>) {
        // FIXME: Can this be done more effciently in rctree,
        // by moving the whole linked list of children at once?
        for child in node.children() {
            new_parent.append(child)
        }
    }

    fn mark_script_already_started(&mut self, _node: NodeRef<'a>) {
        // FIXME: Is this useful outside of a browser?
    }
}
