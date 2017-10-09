use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use std::path::PathBuf;

/// A BookItem corresponds to one entry of the table of contents file SUMMARY.md.
/// A line in that file can either be a numbered chapter with a section number like 2.1.3 or a
/// suffix or postfix chapter without such a section number.
/// The `String` field in the `Chapter` variant contains the section number as `2.1.3`.
/// The `Chapter` type contains the child elements (which can only be other `BookItem::Chapters`).
/// `BookItem::Affix` and `BookItem::Spacer` are only allowed within the root level.
#[derive(Debug, Clone)]
pub enum BookItem {
    Chapter(String, Chapter), // String = section
    Affix(Chapter),
    Spacer,
}

/// A chapter is a `.md` file that is referenced by some line in the `SUMMARY.md` table of
/// contents. It also has references to its sub chapters via `sub_items`. These items can
/// only be of the variant `BookItem::Chapter`.
#[derive(Debug, Clone)]
pub struct Chapter {
    pub name: String,
    pub path: PathBuf,
    pub sub_items: Vec<BookItem>,
}

/// A flattening, depth-first iterator over Bookitems and it's children.
/// It can be obtained by calling `MDBook::iter()`.
#[derive(Debug, Clone)]
pub struct BookItems<'a> {
    /// The remaining items in the iterator in the current, deepest level of the iterator
    items: &'a [BookItem],
    /// The higher levels of the hierarchy. The parents of the current level are still
    /// in the list and accessible as `[stack[0][0], stack[1][0], stack[2][0], ...]`.
    stack: Vec<&'a [BookItem]>,
}

/// Iterator for the parent `BookItem`s of a `BookItem`.
pub struct BookItemParents<'a> {
    stack: &'a [ &'a [BookItem] ]
}

impl Chapter {
    pub fn new(name: String, path: PathBuf) -> Self {
        Chapter {
            name: name,
            path: path,
            sub_items: vec![],
        }
    }
}


impl Serialize for Chapter {
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut struct_ = serializer.serialize_struct("Chapter", 2)?;
        struct_.serialize_field("name", &self.name)?;
        struct_.serialize_field("path", &self.path)?;
        struct_.end()
    }
}

impl<'a> Iterator for BookItems<'a> {
    type Item = &'a BookItem;

    fn next(&mut self) -> Option<&'a BookItem> {
        if let Some((first, rest)) = self.items.split_first() {
            // Return the first element in `items` and optionally dive into afterwards.
            match first {
                &BookItem::Spacer => {
                    self.items = rest;
                },
                &BookItem::Chapter(_, ref ch) |
                &BookItem::Affix(ref ch) => {
                    if ch.sub_items.is_empty() {
                        self.items = rest;
                    } else {
                        // Don't remove `first` for now. (Because of Parent Iterator)
                        self.stack.push(self.items);
                        self.items = &ch.sub_items[..];
                    }
                },
            };
            Some(first)
        } else {
            // Current level is drained => pop from `stack` or return `None`
            if let Some(stacked_items) = self.stack.pop() {
                // The first item of the popped slice is the bookitem we previously dived into.
                self.items = &stacked_items[1..];
                self.next()
            } else {
                None
            }
        }
    }
}

impl<'a> BookItems<'a> {
    pub fn new(items : &'a[BookItem]) -> BookItems<'a> {
        BookItems {
            items : items,
            stack : vec![],
        }
    }

    /// Returns an iterator to iterate the parents of the last yielded `BookItem`.
    /// Starts with the root item.
    pub fn current_parents(&'a self) -> BookItemParents<'a> {
        BookItemParents { stack : &self.stack }
    }

    /// Collects the names of the parent `BookItem`s of the last yielded `Bookitem` into a list.
    pub fn collect_current_parents_names(&self) -> Vec<String> {
        self.current_parents().filter_map(|i| match i {
            &BookItem::Chapter(_, ref ch) | &BookItem::Affix(ref ch) => Some(ch.name.clone()),
            _ => None,
        }).collect()
    }

    /// Get the level of the last yielded `BookItem`. Root level = 0
    pub fn current_depth(&'a self) -> usize {
        self.stack.len()
    }
}

impl<'a> Iterator for BookItemParents<'a> {
    type Item = &'a BookItem;

    fn next(&mut self) -> Option<&'a BookItem> {
        if let Some((first, rest)) = self.stack.split_first() {
            self.stack = rest;
            Some (&first[0])
        } else {
            None
        }
    }
}