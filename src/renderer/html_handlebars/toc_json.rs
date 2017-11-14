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

fn strip_name(name: &str) -> String {
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

    markdown_parsed_name
}

/// Create a json node for a given chapter.
fn make_chapter_node(item: &BTreeMap<String, String>) -> serde_json::Value {
    let mut map = BTreeMap::new();

    if let Some(path) = item.get("path") {
        if !path.is_empty() {
            map.insert("link".to_owned(), json!(path_to_link(path)));
        }
    }

    if let Some(section) = item.get("section") {
        map.insert("section".to_owned(), json!(section));
    }

    if let Some(name) = item.get("name") {
        map.insert("name".to_owned(), json!(strip_name(name)));
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

    json!(map)
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
        let node = if item.get("spacer").is_some() {
            // Spacers never belong to a section
            extend_or_collapse_path(1, &mut path);

            json!({"spacer": true})
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

            make_chapter_node(item)
        };

        path.push(node);
    }

    extend_or_collapse_path(1, &mut path);

    Ok(path.pop().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_toc_tree() {
        let expected_toc = json!({
            "children": [
            {
                "name": "Introduction",
                "section": "1.",
                "link": "intro.html",
                "next": {
                    "link": "start.html"
                },
                "children": [
                {
                    "name": "Getting started",
                    "section": "1.1.",
                    "link": "start.html",
                    "previous": {
                        "link": "intro.html"
                    },
                    "next": {
                        "link": "indepth.html"
                    }
                }
                ]
            },
            {
                "name": "In depth",
                "section": "2.",
                "link": "indepth.html",
                "previous": {
                    "link": "start.html"
                }
            }
            ]
        });

        let mut chapter1 = BTreeMap::new();
        chapter1.insert("name".to_owned(), "Introduction".to_owned());
        chapter1.insert("section".to_owned(), "1.".to_owned());
        chapter1.insert("path".to_owned(), "intro".to_owned());
        chapter1.insert("next_path".to_owned(), "start".to_owned());

        let mut chapter2 = BTreeMap::new();
        chapter2.insert("name".to_owned(), "Getting started".to_owned());
        chapter2.insert("section".to_owned(), "1.1.".to_owned());
        chapter2.insert("path".to_owned(), "start".to_owned());
        chapter2.insert("next_path".to_owned(), "indepth".to_owned());
        chapter2.insert("previous_path".to_owned(), "intro".to_owned());

        let mut chapter3 = BTreeMap::new();
        chapter3.insert("name".to_owned(), "In depth".to_owned());
        chapter3.insert("section".to_owned(), "2.".to_owned());
        chapter3.insert("path".to_owned(), "indepth".to_owned());
        chapter3.insert("previous_path".to_owned(), "start".to_owned());

        let chapters = vec![chapter1, chapter2, chapter3];
        let toc = make_toc_tree(chapters.as_slice()).unwrap();

        assert_eq!(toc, expected_toc);
    }
}
