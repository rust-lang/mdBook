//! The datatypes used to describe a `Book` in memory.

use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use errors::*;


/// An in-memory representation of the entire book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub sections: Vec<BookItem>,
}

/// Any item which the book may contain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BookItem {
    Chapter(String, Chapter),
    Affix(Chapter),
    Spacer,
}

/// A single chapter, which may or may not have sub-chapters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    /// The chapter name as specified in the `SUMMARY.md`.
    pub name: String,
    /// The file's location relative to the project root.
    pub path: PathBuf,
    /// The chapter's raw text.
    pub contents: String,
    /// Any sub-items in the chapter.
    pub items: Vec<BookItem>,
}

impl Chapter {
    pub fn new(name: String, path: PathBuf) -> Result<Chapter> {
        let mut contents = String::new();
        File::open(&path)?.read_to_string(&mut contents)?;

        Ok(Chapter {
            name: name,
            path: path,
            contents: contents,
            items: Vec::new(),
        })
    }
}

/// Book walker implemented using the [Visitor Pattern] for performing
/// manipulations on a `Book`.
///
/// Each method of a `Visitor` is a hook which can potentially be overridden,
/// with the default methods simply recursively visiting each node in a `Book`
/// (e.g. the visit_section method by default calls visit::walk_section).
///
/// Each overridden visit method has full control over what happens with its
/// node, it can do its own traversal of the node's children, call visit::walk_*
/// to apply the default traversal algorithm, or prevent deeper traversal by
/// doing nothing.
///
/// > **Note:** The idea for implementing this was shamelessly stolen from
/// [syn].
///
/// [syn]: https://docs.serde.rs/syn/visit/trait.Visitor.html
/// [Visitor Pattern]: https://en.wikipedia.org/wiki/Visitor_pattern
pub trait Visitor: Sized {
    fn visit_book(&mut self, book: &mut Book) {
        visit::walk_book(self, book);
    }
    fn visit_section(&mut self, section: &mut BookItem) {
        visit::walk_section(self, section);
    }
    fn visit_chapter(&mut self, ch: &mut Chapter) {
        visit::walk_chapter(self, ch);
    }
}


/// Helper functions which may be called by a `Visitor` to continue the default
/// traversal.
pub mod visit {
    use super::{Chapter, Book, BookItem, Visitor};

    /// A function a `Visitor` may call to make sure the rest of the `Book` gets
    /// visited.
    pub fn walk_book<V: Visitor>(visitor: &mut V, book: &mut Book) {
        for section in book.sections.iter_mut() {
            visitor.visit_section(section);
        }
    }

    /// A function a `Visitor` may call to make sure the `Chapter` inside this
    /// `BookItem` (if there is one) gets visited.
    pub fn walk_section<V: Visitor>(visitor: &mut V, section: &mut BookItem) {
        match *section {
            BookItem::Chapter(_, ref mut ch) |
            BookItem::Affix(ref mut ch) => visitor.visit_chapter(ch),
            _ => {},
        }
    }

    /// A function a `Visitor` may call to make sure the rest of the items in a
    /// `Chapter` get visited.
    pub fn walk_chapter<V: Visitor>(visitor: &mut V, chapter: &mut Chapter) {
        for item in chapter.items.iter_mut() {
            visitor.visit_section(item);
        }
    }
}
