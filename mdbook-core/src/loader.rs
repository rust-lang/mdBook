//! Functionality for inspecting the source for a book and generating an
//! in-memory representation.

#![allow(unused_variables, dead_code)]

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

use config::{load_config, Config};
use book::{Chapter, BookItem};
use errors::*;

/// Loader is the object in charge of loading the source documents from disk.
///
/// It Will:
///
/// - Initialize a new project
/// - Parse `SUMMARY.md`
/// - Traverse the source directory, looking for markdown files
/// - Turn all of that into a single data structure which is an in-memory
///   representation of the book
#[derive(Clone, Debug, PartialEq)]
pub struct Loader {
    root: PathBuf,
    config: Config,
}

impl Loader {
    /// Create a new `Loader` with `root` as the book's root directory.
    ///
    /// # Note
    ///
    /// This constructor will automatically parse the config file, so it may
    /// fail if the config file doesn't exist or is corrupted.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Loader> {
        let root = PathBuf::from(root.as_ref());

        let config = load_config(&root).chain_err(
            || "Couldn't load the config file",
        )?;
        Ok(Loader {
            root: root,
            config: config,
        })
    }

    fn summary_toml(&self) -> PathBuf {
        self.root.join(self.config.source_directory()).join(
            "SUMMARY.md",
        )
    }

    fn parse_summary(&self) -> Result<Summary> {
        let mut summary = String::new();
        File::open(self.summary_toml())
            .chain_err(|| "Couldn't open the SUMMARY.toml")?
            .read_to_string(&mut summary)?;

        // TODO: The existing `parse_level()` function needs to be adapted to the new
        // types
        unimplemented!()
    }
}


#[derive(Clone, Debug, Default, PartialEq)]
struct Summary {
    /// The summary items and which "level" they are on
    items: Vec<SummaryItem>,
}

#[derive(Clone, Debug, PartialEq)]
enum SummaryItem {
    /// A chapter containing its name, "level", and the location on disk.
    Chapter(String, PathBuf),
    Affix(String, PathBuf),
    Spacer,
}

// const CORRUPTED_SUMMARY_ERROR_MSG: &str = "Your summary.md is messed up.
//
// Prefix, Suffix and Spacer elements can only exist on the root level. Prefix
// elements can only exist before any chapter and there can be no chapters after
// suffix elements.
// ";

// /// Recursively parse each level in the `SUMMARY.md`, constructing the
// `BookItems`
// /// as you go.
// fn parse_level(summary: &mut Vec<&str>, current_level: i32, mut section:
// Vec<i32>) -> Result<Vec<SummaryItem>> {
// // FIXME: Return an in-memory representation of the summary instead of
// directly constructing the book
// // At the moment, if you wanted to test *just* the SUMMARY.md parsing,
// you'd
// // need a complete working book on disk. Preferably in a tempdir.
// Ewwww...
//     debug!("[fn]: parse_level");
//     let mut items: Vec<SummaryItem> = Vec::new();

//     // Construct the book recursively
//     while !summary.is_empty() {
//         let item: SummaryItem;

//         // Indentation level of the line to parse
//         let level = level(summary[0], 4)?;

//         // if level < current_level we remove the last digit of section,
//         // exit the current function,
//         // and return the parsed level to the calling function.
//         if level < current_level {
//             break;
//         }

//         // if level > current_level we call ourselves to go one level deeper
//         if level > current_level {
//             // Level can not be root level !!
//             // Add a sub-number to section
//             section.push(0);
//             let last = items.pop()
// .expect("There should be at least one item since this can't
// be the root level");

//             if let SummaryItem::Chapter(ref s, ref ch) = last {
//                 let mut ch = ch.clone();
//                 ch.items = parse_level(summary, level, section.clone())
// .chain_err(|| format!("Couldn't parse level {}",
// level))?;
//                 items.push(SummaryItem::Chapter(s.clone(), ch));

// // Remove the last number from the section, because we got
// back to our level..
//                 section.pop();
//                 continue;
//             } else {
//                 bail!(CORRUPTED_SUMMARY_ERROR_MSG);
//             };

//         } else {
//             // level and current_level are the same, parse the line
//             item = if let Some(parsed_item) = parse_line(summary[0]) {

// // Eliminate possible errors and set section to -1 after
// suffix
//                 match parsed_item {
//                     // error if level != 0 and BookItem is != Chapter
//                     SummaryItem::Affix(_, _) |
//                     SummaryItem::Spacer if level > 0 => {
//                         bail!(CORRUPTED_SUMMARY_ERROR_MSG);
//                     },

//                     // error if BookItem == Chapter and section == -1
//                     SummaryItem::Chapter(_, _) if section[0] == -1 => {
//                         bail!(CORRUPTED_SUMMARY_ERROR_MSG);
//                     },

//                     // Set section = -1 after suffix
//                     SummaryItem::Affix(_, _) if section[0] > 0 => {
//                         section[0] = -1;
//                     },

//                     _ => {},
//                 }

//                 match parsed_item {
//                     SummaryItem::Chapter(_, ch) => {
//                         // Increment section
//                         let len = section.len() - 1;
//                         section[len] += 1;
//                         let s = section.iter()
// .fold("".to_owned(), |s, i| s + &i.to_string() +
// ".");
//                         SummaryItem::Chapter(s, ch)
//                     },
//                     other => other,
//                 }

//             } else {
//                 // If parse_line does not return Some(_) continue...
//                 summary.remove(0);
//                 continue;
//             };
//         }

//         summary.remove(0);
//         items.push(item);
//     }
//     debug!("[*]: Level: {:?}", items);
//     Ok(items)
// }

fn level(line: &str, spaces_in_tab: u32) -> Result<u32> {
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
        debug!("\t[line]: {:?}", line);
        debug!("[*]: There is an indentation error on this line. Indentation should be {} spaces", spaces_in_tab);
        bail!("Indentation error on line:\n\n{}", line);
    }

    Ok(level)
}


/// Parse a single line and figure out what kind of item it is.
fn parse_line(l: &str) -> Option<SummaryItem> {
    debug!("[fn]: parse_line");

    // Remove leading and trailing spaces or tabs
    let line = l.trim_matches(|c: char| c == ' ' || c == '\t');

    // Spacers are "------"
    if line.starts_with("--") {
        debug!("[*]: Line is spacer");
        return Some(SummaryItem::Spacer);
    }

    if let Some(c) = line.chars().nth(0) {
        match c {
            // List item
            '-' | '*' => {
                debug!("[*]: Line is list element");

                if let Some((name, path)) = read_link(line) {
                    return Some(SummaryItem::Chapter(name, path));
                } else {
                    return None;
                }
            },
            // Non-list element
            '[' => {
                debug!("[*]: Line is a link element");

                if let Some((name, path)) = read_link(line) {
                    return Some(SummaryItem::Affix(name, path));
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


#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;
    use std::fs;

    /// A crappy `cp -r` clone
    fn copy_dir<P, Q>(from: P, to: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        assert!(from.as_ref().exists());

        let to = to.as_ref();
        fs::create_dir_all(to)?;

        for entry in from.as_ref().read_dir()? {
            let original = entry?.path();
            let name = original
                .file_name()
                .expect("Files in a directory must have a name")
                .to_str()
                .unwrap();
            let new_path = to.join(name);

            if original.is_file() {
                fs::copy(&original, new_path)?;
            } else if original.is_dir() {
                copy_dir(&original, new_path)?;
            }
        }

        Ok(())
    }

    fn new_book_directory() -> TempDir {
        let book_example_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("book-example");

        let temp = TempDir::new("book-example").unwrap();
        copy_dir(&book_example_dir, temp.path()).unwrap();

        temp
    }

    #[test]
    fn read_lines() {
        let inputs = vec![
            ("[First](first.md)", Some(("First", "first.md"))),
            ("[First]", None),
            ("[First][second.md]", None),
            ("other stuff", None),
            ("- [dot point](dot_point.md)", Some(("dot point", "dot_point.md"))),
        ];

        for (src, should_be) in inputs {
            let got = read_link(src);

            let should_be = should_be.map(|s| (s.0.to_string(), PathBuf::from(s.1)));
            assert_eq!(got, should_be);
        }
    }

    /// This checks that the SUMMARY.md parser can correctly parse the
    /// SUMMARY.md
    /// in `book-example`. This should help to prevent regression bugs.
    #[test]
    #[ignore]
    fn parse_book_example_summary() {
        // TODO: remove the `#[ignore]` when Loader::parse_summary() is implemented
        let temp = new_book_directory();
        let loader = Loader::new(temp.path()).unwrap();

        let summary = loader.parse_summary().unwrap();
    }
}
