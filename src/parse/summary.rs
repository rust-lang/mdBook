use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Result, Error, ErrorKind};

use book::chapter::Chapter;
use book::toc::{TocItem, TocContent};

pub fn construct_tocitems(summary_path: &PathBuf, first_as_index: bool) -> Result<Vec<TocItem>> {
    debug!("[fn]: construct_tocitems");
    let mut summary = String::new();
    try!(try!(File::open(summary_path)).read_to_string(&mut summary));

    debug!("[*]: Parse SUMMARY.md");

    let top_items = try!(parse_level(&mut summary.split('\n').collect(), 0, vec![0], first_as_index));
    debug!("[*]: Done parsing SUMMARY.md");
    Ok(top_items)
}

pub fn parse_level(summary: &mut Vec<&str>, current_level: i32, mut section: Vec<i32>, first_as_index: bool) -> Result<Vec<TocItem>> {
    debug!("[fn]: parse_level");
    let mut items: Vec<TocItem> = vec![];

    let mut found_first = false;

    let ohnoes = r#"Your SUMMARY.md is messed up

Unnumbered and Spacer items can only exist on the root level.

Unnumbered items can only exist before or after Numbered items, since these
items are in the frontmatter of a book.

There can be no Numbered items after Unnumbered items, as they are in the
backmatter."#;

    // Construct the book recursively
    while !summary.is_empty() {
        let item: TocItem;
        // Indentation level of the line to parse
        let level = try!(level(summary[0], 4));

        // if level < current_level we remove the last digit of section, exit the current function,
        // and return the parsed level to the calling function.
        if level < current_level {
            break;
        }

        // if level > current_level we call ourselves to go one level deeper
        if level > current_level {
            // Level can not be root level !!
            // Add a sub-number to section
            section.push(0);

            let last = items.pop().expect("There should be at least one item since this can't be the root level");

            item = match last {
                TocItem::Numbered(mut a) => {
                    let sec = section.clone();
                    a.sub_items = Some(try!(parse_level(summary, level, sec.clone(), false)));
                    items.push(TocItem::Numbered(a));

                    // Remove the last number from the section, because we got
                    // back to our level...
                    section.pop();
                    continue;
                },
                TocItem::Unnumbered(mut a) => {
                    let sec = section.clone();
                    a.sub_items = Some(try!(parse_level(summary, level, sec.clone(), false)));
                    items.push(TocItem::Unnumbered(a));
                    section.pop();
                    continue;
                },
                TocItem::Unlisted(mut a) => {
                    let sec = section.clone();
                    a.sub_items = Some(try!(parse_level(summary, level, sec.clone(), false)));
                    items.push(TocItem::Unlisted(a));
                    section.pop();
                    continue;
                },
                _ => {
                    return Err(Error::new(ErrorKind::Other, ohnoes));
                }
            };

        } else {
            // level and current_level are the same, parse the line
            item = if let Some(parsed_item) = parse_line(summary[0]) {

                // Eliminate possible errors and set section to -1 after unnumbered
                match parsed_item {

                    // error if level != 0 and TocItem is != Numbered
                    TocItem::Unnumbered(_) | TocItem::Spacer if level > 0 => {
                        return Err(Error::new(ErrorKind::Other, ohnoes))
                    },

                    // error if TocItem == Numbered or Unlisted and section == -1
                    TocItem::Numbered(_) | TocItem::Unlisted(_) if section[0] == -1 => {
                        return Err(Error::new(ErrorKind::Other, ohnoes))
                    },

                    // Set section = -1 after unnumbered
                    TocItem::Unnumbered(_) if section[0] > 0 => {
                        section[0] = -1;
                    },

                    _ => {},
                }

                match parsed_item {
                    TocItem::Numbered(mut content) => {
                        // Increment section
                        let len = section.len() - 1;
                        section[len] += 1;

                        content.section = Some(section.clone());

                        TocItem::Numbered(content)
                    },
                    _ => parsed_item,
                }

            } else {
                // If parse_line does not return Some(_) continue...
                summary.remove(0);
                continue;
            };
        }

        summary.remove(0);

        if first_as_index && !found_first {
            let i = match item {
                TocItem::Numbered(mut content) => {
                    found_first = true;
                    content.chapter.set_dest_path(PathBuf::from("index.html".to_string()));
                    TocItem::Numbered(content)
                },
                TocItem::Unnumbered(mut content) => {
                    found_first = true;
                    content.chapter.set_dest_path(PathBuf::from("index.html".to_string()));
                    TocItem::Unnumbered(content)
                },
                TocItem::Unlisted(content) => {
                    TocItem::Unlisted(content)
                },
                TocItem::Spacer => TocItem::Spacer,
            };
            items.push(i);
        } else {
            items.push(item);
        }
    }
    debug!("[*]: Level: {:?}", items);
    Ok(items)
}

fn level(line: &str, spaces_in_tab: i32) -> Result<i32> {
    debug!("[fn]: level");
    let mut spaces = 0;
    let mut level = 0;

    for ch in line.chars() {
        match ch {
            ' ' => spaces += 1,
            '\t' => level += 1,
            _ => break,
        }
        if spaces >= spaces_in_tab {
            level += 1;
            spaces = 0;
        }
    }

    // If there are spaces left, there is an indentation error
    if spaces > 0 {
        debug!("[SUMMARY.md]:");
        debug!("\t[line]: {}", line);
        debug!("[*]: There is an indentation error on this line. Indentation should be {} spaces", spaces_in_tab);
        return Err(Error::new(ErrorKind::Other, format!("Indentation error on line:\n\n{}", line)));
    }

    Ok(level)
}

fn parse_line(l: &str) -> Option<TocItem> {
    debug!("[fn]: parse_line");

    // Remove leading and trailing spaces or tabs
    let line = l.trim_matches(|c: char| c == ' ' || c == '\t');

    // Spacers are "------"
    if line.starts_with("--") {
        debug!("[*]: Line is spacer");
        return Some(TocItem::Spacer);
    }

    if let Some(c) = line.chars().nth(0) {
        match c {
            // List item
            '-' | '*' => {
                debug!("[*]: Line is list element");

                if let Some((title, path)) = read_link(line) {
                    let chapter = Chapter::new(title, path);
                    return Some(TocItem::Numbered(TocContent::new(chapter)));
                } else {
                    return None;
                }
            },
            // Non-list element
            '[' => {
                debug!("[*]: Line is a link element");

                if let Some((title, path)) = read_link(line) {
                    let chapter = Chapter::new(title, path);
                    return Some(TocItem::Unnumbered(TocContent::new(chapter)));
                } else {
                    return None;
                }
            },
            _ => {},
        }
    }

    None
}

fn read_link(line: &str) -> Option<(String, PathBuf)> {
    let mut start_delimitor;
    let mut end_delimitor;

    // In the future, support for list item that is not a link
    // Not sure if I should error on line I can't parse or just ignore them...
    if let Some(i) = line.find('[') {
        start_delimitor = i;
    } else {
        debug!("[*]: '[' not found, this line is not a link. Ignoring...");
        return None;
    }

    if let Some(i) = line[start_delimitor..].find("](") {
        end_delimitor = start_delimitor + i;
    } else {
        debug!("[*]: '](' not found, this line is not a link. Ignoring...");
        return None;
    }

    let title = line[start_delimitor + 1..end_delimitor].to_owned();

    start_delimitor = end_delimitor + 1;
    if let Some(i) = line[start_delimitor..].find(')') {
        end_delimitor = start_delimitor + i;
    } else {
        debug!("[*]: ')' not found, this line is not a link. Ignoring...");
        return None;
    }

    let path = PathBuf::from(line[start_delimitor + 1..end_delimitor].to_owned());

    Some((title, path))
}
