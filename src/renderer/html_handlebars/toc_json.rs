use std::cmp;
use std::collections::BTreeMap;
use std::path::Path;

use pulldown_cmark::{html, Parser, Event, Tag};
use serde_json;
use errors::*;

fn path_to_link(path: &str) -> String {
    Path::new(path)
        .with_extension("html")
        .to_str()
        .unwrap()
        // Hack for windows who tends to use `\` as separator instead of `/`
        .replace("\\", "/")
}

/// Set name etc for a chapter.
fn set_props(map: &mut BTreeMap<String, serde_json::Value>, item: &BTreeMap<String, String>) {
    if let Some(path) = item.get("path") {
        if !path.is_empty() {
            map.insert("link".to_owned(), json!(path_to_link(path)));
        }
    }

    // Section does not necessarily exist
    if let Some(section) = item.get("section") {
        map.insert("section".to_owned(), json!(section));
    }

    if let Some(name) = item.get("name") {
        // filter all events that are not inline code blocks
        let parser = Parser::new(name).filter(|event| match *event {
                                                  Event::Start(Tag::Code) |
                                                  Event::End(Tag::Code) |
                                                  Event::InlineHtml(_) |
                                                  Event::Text(_) => true,
                                                  _ => false,
                                              });

        // render markdown to html
        let mut markdown_parsed_name = String::with_capacity(name.len() * 3 / 2);
        html::push_html(&mut markdown_parsed_name, parser);

        map.insert("name".to_owned(), json!(markdown_parsed_name));
    }

    if let Some(previous_path) = item.get("previous_path") {
        map.insert("previous".to_owned(),
                   json!({
                             "link": path_to_link(previous_path)
                         }));
    }

    if let Some(next_path) = item.get("next_path") {
        map.insert("next".to_owned(),
                   json!({
                             "link": path_to_link(next_path)
                         }));
    }
}

/// Extend or collapse tree path to reach a certain depth.
fn extend_or_collapse_path(level: usize, path: &mut Vec<serde_json::Value>) {
    // Can't pop root node
    assert!(level > 0);

    while level > path.len() {
        path.push(json!({}));
    }

    while level < path.len() {
        // Push child into parent.children
        let child = path.pop().unwrap();
        let parent = path.last_mut().unwrap().as_object_mut().unwrap();

        if !parent.contains_key("children") {
            parent.insert("children".to_owned(), json!([]));
        }

        parent.get_mut("children")
              .unwrap()
              .as_array_mut()
              .unwrap()
              .push(child);
    }
}

/// Turn a flat chapters array into a tree json structure. Each node represents
/// a section and its subsections.
pub fn make_toc_tree(chapters: &[BTreeMap<String, String>]) -> Result<serde_json::Value> {
    // A stack representing the path from the root node to the current node.
    let mut path = vec![];

    for item in chapters {
        let mut current = BTreeMap::new();

        if item.get("spacer").is_some() {
            // Spacers never belong to a section
            extend_or_collapse_path(1, &mut path);
            current.insert("spacer".to_owned(), json!(true));
        } else {
            let level = if let Some(section_name) = item.get("section") {
                // The section "4. Foo" has level 1, "4.1. Bar" has level 2 etc.
                // Unnumbered sections is at the root level 1.
                cmp::max(section_name.matches('.').count(), 1)
            } else {
                // Chapters without sections are also at the root.
                1
            };

            extend_or_collapse_path(level, &mut path);
            set_props(&mut current, item);
        }

        path.push(json!(current));
    }

    extend_or_collapse_path(1, &mut path);

    Ok(path.pop().unwrap())
}
