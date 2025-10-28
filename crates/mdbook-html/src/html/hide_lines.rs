//! Support for hiding code lines.

use crate::html::{Element, Node};
use ego_tree::{NodeId, Tree};
use html5ever::tendril::StrTendril;
use mdbook_core::static_regex;
use std::collections::HashMap;

/// Wraps hidden lines in a `<span>` for the given code block.
pub(crate) fn hide_lines(
    tree: &mut Tree<Node>,
    code_id: NodeId,
    hidelines: &HashMap<String, String>,
) {
    let mut node = tree.get_mut(code_id).unwrap();
    let el = node.value().as_element().unwrap();

    let classes: Vec<_> = el.attr("class").unwrap_or_default().split(' ').collect();
    let language = classes
        .iter()
        .filter_map(|cls| cls.strip_prefix("language-"))
        .next()
        .unwrap_or_default()
        .to_string();
    let hideline_info = classes
        .iter()
        .filter_map(|cls| cls.strip_prefix("hidelines="))
        .map(|prefix| prefix.to_string())
        .next();

    if let Some(mut child) = node.first_child()
        && let Node::Text(text) = child.value()
    {
        if language == "rust" {
            let new_nodes = hide_lines_rust(text);
            child.detach();
            let root = tree.extend_tree(new_nodes);
            let root_id = root.id();
            let mut node = tree.get_mut(code_id).unwrap();
            node.reparent_from_id_append(root_id);
        } else {
            // Use the prefix from the code block, else the prefix from config.
            let hidelines_prefix = hideline_info
                .as_deref()
                .or_else(|| hidelines.get(&language).map(|p| p.as_str()));
            if let Some(prefix) = hidelines_prefix {
                let new_nodes = hide_lines_with_prefix(text, prefix);
                child.detach();
                let root = tree.extend_tree(new_nodes);
                let root_id = root.id();
                let mut node = tree.get_mut(code_id).unwrap();
                node.reparent_from_id_append(root_id);
            }
        }
    }
}

/// Wraps hidden lines in a `<span>` specifically for Rust code blocks.
fn hide_lines_rust(text: &StrTendril) -> Tree<Node> {
    static_regex!(BORING_LINES_REGEX, r"^(\s*)#(.?)(.*)$");

    let mut tree = Tree::new(Node::Fragment);
    let mut root = tree.root_mut();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        // Don't include newline on the last line.
        let newline = if lines.peek().is_none() { "" } else { "\n" };
        if let Some(caps) = BORING_LINES_REGEX.captures(line) {
            if &caps[2] == "#" {
                root.append(Node::Text(
                    format!("{}{}{}{newline}", &caps[1], &caps[2], &caps[3]).into(),
                ));
                continue;
            } else if matches!(&caps[2], "" | " ") {
                let mut span = Element::new("span");
                span.insert_attr("class", "boring".into());
                let mut span = root.append(Node::Element(span));
                span.append(Node::Text(
                    format!("{}{}{newline}", &caps[1], &caps[3]).into(),
                ));
                continue;
            }
        }
        root.append(Node::Text(format!("{line}{newline}").into()));
    }
    tree
}

/// Wraps hidden lines in a `<span>` tag for lines starting with the given prefix.
fn hide_lines_with_prefix(content: &str, prefix: &str) -> Tree<Node> {
    let mut tree = Tree::new(Node::Fragment);
    let mut root = tree.root_mut();
    for line in content.lines() {
        if line.trim_start().starts_with(prefix) {
            let pos = line.find(prefix).unwrap();
            let (ws, rest) = (&line[..pos], &line[pos + prefix.len()..]);
            let mut span = Element::new("span");
            span.insert_attr("class", "boring".into());
            let mut span = root.append(Node::Element(span));
            span.append(Node::Text(format!("{ws}{rest}\n").into()));
        } else {
            root.append(Node::Text(format!("{line}\n").into()));
        }
    }
    tree
}

/// If this code text is missing an `fn main`, the wrap it with `fn main` in a
/// fashion similar to rustdoc, with the wrapper hidden.
pub(crate) fn wrap_rust_main(text: &str) -> Option<String> {
    if !text.contains("fn main") && !text.contains("quick_main!") {
        let (attrs, code) = partition_rust_source(text);
        let newline = if code.is_empty() || code.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        Some(format!(
            "# #![allow(unused)]\n{attrs}# fn main() {{\n{code}{newline}# }}"
        ))
    } else {
        None
    }
}

/// Splits Rust inner attributes from the given source string.
///
/// Returns `(inner_attrs, rest_of_code)`.
fn partition_rust_source(s: &str) -> (&str, &str) {
    static_regex!(
        HEADER_RE,
        r"^(?mx)
        (
            (?:
                ^[ \t]*\#!\[.* (?:\r?\n)?
                |
                ^\s* (?:\r?\n)?
            )*
        )"
    );
    let split_idx = match HEADER_RE.captures(s) {
        Some(caps) => {
            let attributes = &caps[1];
            if attributes.trim().is_empty() {
                // Don't include pure whitespace as an attribute. The
                // whitespace in the regex is intended to handle multiple
                // attributes *separated* by potential whitespace.
                0
            } else {
                attributes.len()
            }
        }
        None => 0,
    };
    s.split_at(split_idx)
}

#[test]
fn it_partitions_rust_source() {
    assert_eq!(partition_rust_source(""), ("", ""));
    assert_eq!(partition_rust_source("let x = 1;"), ("", "let x = 1;"));
    assert_eq!(
        partition_rust_source("fn main()\n{ let x = 1; }\n"),
        ("", "fn main()\n{ let x = 1; }\n")
    );
    assert_eq!(
        partition_rust_source("#![allow(foo)]"),
        ("#![allow(foo)]", "")
    );
    assert_eq!(
        partition_rust_source("#![allow(foo)]\n"),
        ("#![allow(foo)]\n", "")
    );
    assert_eq!(
        partition_rust_source("#![allow(foo)]\nlet x = 1;"),
        ("#![allow(foo)]\n", "let x = 1;")
    );
    assert_eq!(
        partition_rust_source(
            "\n\
        #![allow(foo)]\n\
        \n\
        #![allow(bar)]\n\
        \n\
        let x = 1;"
        ),
        ("\n#![allow(foo)]\n\n#![allow(bar)]\n\n", "let x = 1;")
    );
    assert_eq!(
        partition_rust_source("    // Example"),
        ("", "    // Example")
    );
}
