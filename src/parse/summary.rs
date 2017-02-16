use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Result, Error, ErrorKind};
use book::bookitem::{BookItem, Chapter};

pub fn construct_bookitems(path: &PathBuf) -> Result<Vec<BookItem>> {
    debug!("[fn]: construct_bookitems");
    let mut summary = String::new();
    try!(try!(File::open(path)).read_to_string(&mut summary));

    debug!("[*]: Parse SUMMARY.md");
    let top_items = try!(parse_level(&mut summary.split('\n').collect(), 0, vec![0]));
    debug!("[*]: Done parsing SUMMARY.md");
    Ok(top_items)
}

fn parse_level(summary: &mut Vec<&str>, current_level: i32, mut section: Vec<i32>) -> Result<Vec<BookItem>> {
    debug!("[fn]: parse_level");
    let mut items: Vec<BookItem> = vec![];

    // Construct the book recursively
    while !summary.is_empty() {
        let item: BookItem;
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

            if let BookItem::Chapter(ref s, ref ch) = last {
                let mut ch = ch.clone();
                ch.sub_items = try!(parse_level(summary, level, section.clone()));
                items.push(BookItem::Chapter(s.clone(), ch));

                // Remove the last number from the section, because we got back to our level..
                section.pop();
                continue;
            } else {
                return Err(Error::new(ErrorKind::Other,
                                      "Your summary.md is messed up\n\n
                        Prefix, \
                                       Suffix and Spacer elements can only exist on the root level.\n
                        \
                                       Prefix elements can only exist before any chapter and there can be \
                                       no chapters after suffix elements."));
            };

        } else {
            // level and current_level are the same, parse the line
            item = if let Some(parsed_item) = parse_line(summary[0]) {

                // Eliminate possible errors and set section to -1 after suffix
                match parsed_item {
                    // error if level != 0 and BookItem is != Chapter
                    BookItem::Affix(_) | BookItem::Spacer if level > 0 => {
                        return Err(Error::new(ErrorKind::Other,
                                              "Your summary.md is messed up\n\n
                                \
                                               Prefix, Suffix and Spacer elements can only exist on the \
                                               root level.\n
                                Prefix \
                                               elements can only exist before any chapter and there can be \
                                               no chapters after suffix elements."))
                    },

                    // error if BookItem == Chapter and section == -1
                    BookItem::Chapter(_, _) if section[0] == -1 => {
                        return Err(Error::new(ErrorKind::Other,
                                              "Your summary.md is messed up\n\n
                                \
                                               Prefix, Suffix and Spacer elements can only exist on the \
                                               root level.\n
                                Prefix \
                                               elements can only exist before any chapter and there can be \
                                               no chapters after suffix elements."))
                    },

                    // Set section = -1 after suffix
                    BookItem::Affix(_) if section[0] > 0 => {
                        section[0] = -1;
                    },

                    _ => {},
                }

                match parsed_item {
                    BookItem::Chapter(_, ch) => {
                        // Increment section
                        let len = section.len() - 1;
                        section[len] += 1;
                        let s = section.iter().fold("".to_owned(), |s, i| s + &i.to_string() + ".");
                        BookItem::Chapter(s, ch)
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
        items.push(item)
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


fn parse_line(l: &str) -> Option<BookItem> {
    debug!("[fn]: parse_line");

    // Remove leading and trailing spaces or tabs
    let line = l.trim_matches(|c: char| c == ' ' || c == '\t');

    // Spacers are "------"
    if line.starts_with("--") {
        debug!("[*]: Line is spacer");
        return Some(BookItem::Spacer);
    }

    if let Some(c) = line.chars().nth(0) {
        match c {
            // List item
            '-' | '*' => {
                debug!("[*]: Line is list element");

                if let Some((name, path)) = read_link(line) {
                    return Some(BookItem::Chapter("0".to_owned(), Chapter::new(name, path)));
                } else {
                    return None;
                }
            },
            // Non-list element
            '[' => {
                debug!("[*]: Line is a link element");

                if let Some((name, path)) = read_link(line) {
                    return Some(BookItem::Affix(Chapter::new(name, path)));
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

    let name = line[start_delimitor + 1..end_delimitor].to_owned();

    start_delimitor = end_delimitor + 1;
    if let Some(i) = line[start_delimitor..].find(')') {
        end_delimitor = start_delimitor + i;
    } else {
        debug!("[*]: ')' not found, this line is not a link. Ignoring...");
        return None;
    }

    let path = PathBuf::from(line[start_delimitor + 1..end_delimitor].to_owned());

    Some((name, path))
}
