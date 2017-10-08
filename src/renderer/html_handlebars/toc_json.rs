use std::collections::BTreeMap;
use std::path::Path;

use pulldown_cmark::{html, Parser, Event, Tag};
use serde_json;
use errors::*;

/// Set name etc for a chapter.
fn set_props(map: &mut BTreeMap<String, serde_json::Value>, item: &BTreeMap<String, String>) {
    if let Some(path) = item.get("path") {
        if !path.is_empty() {
            let tmp = Path::new(item.get("path").expect("Error: path should be Some(_)"))
                .with_extension("html")
                .to_str()
                .unwrap()
                // Hack for windows who tends to use `\` as separator instead of `/`
                .replace("\\", "/");

            map.insert("link".to_owned(), json!(tmp));
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
}

/// Extend or collapse levels to reach a certain depth.
fn set_level(level: usize, levels: &mut Vec<serde_json::Value>) {
    // Can't pop root node
    assert!(level > 0);

    while level > levels.len() {
        levels.push(json!({}));
    }

    while level < levels.len() {
        // Push child into parent.children
        let child = levels.pop().unwrap();
        let mut parent = levels.last_mut()
            .unwrap()
            .as_object_mut()
            .unwrap();

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

pub fn from_chapters(chapters: &[BTreeMap<String, String>]) -> Result<serde_json::Value> {
    let mut levels = vec![];

    for item in chapters {
        let mut current = BTreeMap::new();

        if item.get("spacer").is_some() {
            set_level(1, &mut levels);
            current.insert("spacer".to_owned(), json!(true));
        } else {
            let level = if let Some(s) = item.get("section") {
                ::std::cmp::max(s.matches('.').count(), 1)
            } else {
                1
            };

            set_level(level, &mut levels);
            set_props(&mut current, item);
        }

        levels.push(json!(current));
    }

    set_level(1, &mut levels);

    return Ok(levels.pop().unwrap());
}
