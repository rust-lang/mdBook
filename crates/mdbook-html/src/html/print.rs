//! Support for generating the print page.
//!
//! The print page takes all the individual chapters (as `Tree<Node>`
//! elements) and modifies the chapters so that they work on a consolidated
//! print page, and then serializes it all as one HTML page.

use super::Node;
use crate::html::{ChapterTree, Element, serialize};
use crate::utils::{ToUrlPath, id_from_content, normalize_path, unique_id};
use mdbook_core::static_regex;
use std::collections::{HashMap, HashSet};
use std::path::{Component, PathBuf};

/// Takes all the chapter trees, modifies them to be suitable to render for
/// the print page, and returns an string of all the chapters rendered to a
/// single HTML page.
pub(crate) fn render_print_page(mut chapter_trees: Vec<ChapterTree<'_>>) -> String {
    let (id_remap, mut id_counter) = make_ids_unique(&mut chapter_trees);
    let path_to_root_id = make_root_id_map(&mut chapter_trees, &mut id_counter);
    rewrite_links(&mut chapter_trees, &id_remap, &path_to_root_id);

    let mut print_content = String::new();
    for ChapterTree { tree, .. } in chapter_trees {
        if !print_content.is_empty() {
            // Add page break between chapters
            // See https://developer.mozilla.org/en-US/docs/Web/CSS/break-before and https://developer.mozilla.org/en-US/docs/Web/CSS/page-break-before
            // Add both two CSS properties because of the compatibility issue
            print_content
                .push_str(r#"<div style="break-before: page; page-break-before: always;"></div>"#);
        }
        serialize(&tree, &mut print_content);
    }
    print_content
}

/// Make all IDs unique, and create a map from old to new IDs.
///
/// The first map is a map of the chapter path to the IDs that were rewritten
/// in that chapter (old ID to new ID).
///
/// The second map is a map of every ID seen to the number of times it has
/// been seen. This is used to generate unique IDs.
fn make_ids_unique(
    chapter_trees: &mut [ChapterTree<'_>],
) -> (HashMap<PathBuf, HashMap<String, String>>, HashSet<String>) {
    let mut id_remap = HashMap::new();
    let mut id_counter = HashSet::new();
    for ChapterTree {
        html_path, tree, ..
    } in chapter_trees
    {
        for value in tree.values_mut() {
            if let Node::Element(el) = value
                && let Some(id) = el.attr("id")
            {
                let new_id = unique_id(id, &mut id_counter);
                if new_id != id {
                    let id = id.to_string();
                    el.insert_attr("id", new_id.clone().into());

                    let map: &mut HashMap<_, _> = id_remap.entry(html_path.clone()).or_default();
                    map.insert(id, new_id);
                }
            }
        }
    }
    (id_remap, id_counter)
}

/// Generates a map of a chapter path to the ID of the top of the chapter.
///
/// If a chapter is missing an `h1` tag, then one is synthesized so that the
/// print output has something to link to.
fn make_root_id_map(
    chapter_trees: &mut [ChapterTree<'_>],
    id_counter: &mut HashSet<String>,
) -> HashMap<PathBuf, String> {
    let mut path_to_root_id = HashMap::new();
    for ChapterTree {
        chapter,
        html_path,
        tree,
        ..
    } in chapter_trees
    {
        let mut h1_found = false;
        for value in tree.values_mut() {
            if let Node::Element(el) = value {
                if el.name() == "h1" {
                    if let Some(id) = el.attr("id") {
                        h1_found = true;
                        path_to_root_id.insert(html_path.clone(), id.to_string());
                    }
                    break;
                } else if matches!(el.name(), "h2" | "h3" | "h4" | "h5" | "h6") {
                    // h1 not found.
                    break;
                }
            }
        }
        if !h1_found {
            // Synthesize a root id to be able to link to the start of the page.
            // TODO: This might want to be a warning? Chapters generally
            // should start with an h1.
            let mut h1 = Element::new("h1");
            let id = id_from_content(&chapter.name);
            let id = unique_id(&id, id_counter);
            h1.insert_attr("id", id.clone().into());
            let mut root = tree.root_mut();
            let mut h1 = root.prepend(Node::Element(h1));
            let mut a = Element::new("a");
            a.insert_attr("href", format!("#{id}").into());
            a.insert_attr("class", "header".into());
            let mut a = h1.append(Node::Element(a));
            a.append(Node::Text(chapter.name.clone().into()));
            path_to_root_id.insert(html_path.clone(), id);
        }
    }

    path_to_root_id
}

/// Rewrite links so that they point to IDs on the print page.
fn rewrite_links(
    chapter_trees: &mut [ChapterTree<'_>],
    id_remap: &HashMap<PathBuf, HashMap<String, String>>,
    path_to_root_id: &HashMap<PathBuf, String>,
) {
    static_regex!(
        LINK,
        r"(?x)
            (?P<scheme>^[a-z][a-z0-9+.-]*:)?
            (?P<path>[^\#]+)?
            (?:\#(?P<anchor>.*))?"
    );

    // Rewrite path links to go to the appropriate place.
    for ChapterTree {
        html_path, tree, ..
    } in chapter_trees
    {
        let base = html_path.parent().expect("path can't be empty");

        for value in tree.values_mut() {
            let Node::Element(el) = value else {
                continue;
            };
            if !matches!(el.name(), "a" | "img") {
                continue;
            }
            for attr in ["href", "src", "xlink:href"] {
                let Some(dest) = el.attr(attr) else {
                    continue;
                };
                let Some(caps) = LINK.captures(&dest) else {
                    continue;
                };
                if caps.name("scheme").is_some() {
                    continue;
                }
                // The lookup_key is the key to look up in the remap table.
                let mut lookup_key = html_path.clone();
                if let Some(href_path) = caps.name("path")
                    && let href_path = href_path.as_str()
                    && !href_path.is_empty()
                {
                    lookup_key.pop();
                    lookup_key.push(href_path);
                    let normalized = normalize_path(&lookup_key);
                    // If this points outside of the book, don't modify it.
                    let is_outside = matches!(
                        normalized.components().next(),
                        Some(Component::ParentDir | Component::RootDir)
                    );
                    if is_outside || !href_path.ends_with(".html") {
                        // Make the link relative to the print page location.
                        let mut rel_path = normalize_path(&base.join(href_path)).to_url_path();
                        if let Some(anchor) = caps.name("anchor") {
                            rel_path.push('#');
                            rel_path.push_str(anchor.as_str());
                        }
                        el.insert_attr(attr, rel_path.into());
                        continue;
                    }
                }

                let lookup_key = normalize_path(&lookup_key);

                let anchor = caps.name("anchor");
                let id = match anchor {
                    Some(anchor_id) => {
                        let anchor_id = anchor_id.as_str().to_string();
                        match id_remap.get(&lookup_key) {
                            Some(id_map) => match id_map.get(&anchor_id) {
                                Some(new_id) => new_id.clone(),
                                None => anchor_id,
                            },
                            None => {
                                // Assume the anchor goes to some non-remapped
                                // ID that already exists.
                                anchor_id
                            }
                        }
                    }
                    None => match path_to_root_id.get(&lookup_key) {
                        Some(id) => id.to_string(),
                        None => continue,
                    },
                };
                el.insert_attr(attr, format!("#{id}").into());
            }
        }
    }
}
