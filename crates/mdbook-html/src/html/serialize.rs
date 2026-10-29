//! Serializes the [`Node`] tree to an HTML string.

use super::tree::is_void_element;
use super::tree::{Element, Node};
use ego_tree::{Tree, iter::Edge};
use html5ever::{local_name, ns};
use mdbook_core::utils::{escape_html, escape_html_attribute};
use std::ops::Deref;

/// Serializes the given tree of [`Node`] elements to an HTML string.
pub(crate) fn serialize(tree: &Tree<Node>, output: &mut String) {
    for edge in tree.root().traverse() {
        match edge {
            Edge::Open(node) => match node.value() {
                Node::Element(el) => serialize_start(el, output),
                Node::Text(text) => {
                    output.push_str(&escape_html(text));
                }
                Node::Comment(comment) => {
                    output.push_str("<!--");
                    output.push_str(comment);
                    output.push_str("-->");
                }
                Node::Fragment => {}
                Node::RawData(html) => {
                    output.push_str(html);
                }
            },
            Edge::Close(node) => {
                if let Node::Element(el) = node.value() {
                    serialize_end(el, output);
                }
            }
        }
    }
}

/// Returns true if this HTML element wants a newline to keep the emitted
/// output more readable.
fn wants_pretty_html_newline(name: &str) -> bool {
    matches!(name, |"blockquote"| "dd"
        | "div"
        | "dl"
        | "dt"
        | "h1"
        | "h2"
        | "h3"
        | "h4"
        | "h5"
        | "h6"
        | "hr"
        | "li"
        | "ol"
        | "p"
        | "pre"
        | "table"
        | "tbody"
        | "thead"
        | "tr"
        | "ul")
}

/// Emit the start tag of an element.
fn serialize_start(el: &Element, output: &mut String) {
    let el_name = el.name();
    if wants_pretty_html_newline(el_name) {
        if !output.is_empty() {
            if !output.ends_with('\n') {
                output.push('\n');
            }
        }
    }
    output.push('<');
    output.push_str(el_name);
    for (attr_name, value) in &el.attrs {
        output.push(' ');
        match attr_name.ns {
            ns!() => (),
            ns!(xml) => output.push_str("xml:"),
            ns!(xmlns) => {
                if el.name.local != local_name!("xmlns") {
                    output.push_str("xmlns:");
                }
            }
            ns!(xlink) => output.push_str("xlink:"),
            _ => (), // TODO what should it do here?
        }
        output.push_str(attr_name.local.deref());
        output.push_str("=\"");
        output.push_str(&escape_html_attribute(&value));
        output.push('"');
    }
    if el.self_closing {
        output.push_str(" /");
    }
    output.push('>');
}

/// Emit the end tag of an element.
fn serialize_end(el: &Element, output: &mut String) {
    // Void elements do not have an end tag.
    if el.self_closing || is_void_element(el.name()) {
        return;
    }
    let name = el.name();
    output.push_str("</");
    output.push_str(name);
    output.push('>');
    if wants_pretty_html_newline(name) {
        output.push('\n');
    }
}
