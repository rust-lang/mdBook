use std::fs::File;
use std::path::{Path, PathBuf};

use book::bookconfig::BookConfig;
use book::toc::{TocItem, TocContent};

use utils::fs::create_with_str;
use parse::construct_tocitems;

/// The `Book` struct contains the metadata (config) and chapters (toc) for one
/// language of the book. Multiple `Book` structs are combined in the `MDBook`
/// struct to support multi-language books.
#[derive(Debug, Clone)]
pub struct Book {
    pub config: BookConfig,
    pub toc: Vec<TocItem>,
}

impl Default for Book {
    fn default() -> Book {
        Book {
            config: BookConfig::default(),
            toc: vec![],
        }
    }
}

impl Book {

    /// Creates a new book
    pub fn new(project_root: &PathBuf) -> Book {
        let conf = BookConfig::new(project_root);
        let mut book = Book::default();
        book.config = conf;
        book
    }

    /// Parses in the SUMMARY.md or creates one
    pub fn parse_or_create_summary_file(&mut self, first_as_index: bool) -> Result<&mut Self, String> {

        let summary_path = self.config.src.join("SUMMARY.md");
        if !summary_path.exists() {
            try!(create_with_str(&summary_path, "# Summary"));
        }

        // parse SUMMARY.md to toc items
        self.toc = match construct_tocitems(&summary_path, first_as_index) {
            Ok(x) => x,
            Err(e) => { return Err(format!("Error constructing the TOC: {:?}", e)); }
        };

        Ok(self)
    }

    /// Walks through the TOC array and calls parse_or_create() on each
    pub fn parse_or_create_chapter_files(&mut self) -> Result<&mut Self, String> {
        self.toc = self.process_them(&self.toc);
        Ok(self)
    }

    fn process_them(&self, items: &Vec<TocItem>) -> Vec<TocItem> {
        items.iter().map(|i|
                         match i {
                             &TocItem::Numbered(ref c) => TocItem::Numbered(self.process_toccontent(c)),
                             &TocItem::Unnumbered(ref c) => TocItem::Unnumbered(self.process_toccontent(c)),
                             &TocItem::Unlisted(ref c) => TocItem::Unlisted(self.process_toccontent(c)),
                             &TocItem::Spacer => TocItem::Spacer,
                         }
        ).collect::<Vec<TocItem>>()
    }

    fn process_toccontent(&self, c: &TocContent) -> TocContent {
        let mut content: TocContent = c.clone();
        if let Ok(ch) = content.chapter.clone().parse_or_create_using(&self.config.src) {
            content.chapter = ch.to_owned();
        }
        if let Some(s) = content.sub_items {
            let subs = self.process_them(&s);
            content.sub_items = Some(subs);
        }
        content
    }

    // TODO update

    // /// This method takes a slice `&[x, y, z]` as parameter and returns the corresponding chapter.
    // /// For example, to retrieve chapter 2.3 we would use:
    // /// ```
    // /// #extern crate mdbook;
    // /// #use mdbook::book::Book;
    // /// #fn main() {
    // /// #let book = Book::new("Test");
    // /// let chapter_2_3 = book.get_chapter(&[2, 3]);
    // /// #}
    // /// ```
    // pub fn get_chapter(&self, section: &[usize]) -> Option<&Chapter> {
    //     match section.len() {
    //         0 => None,
    //         1 => self.mainmatter.get(section[0]),
    //         _ => {
    //             self.mainmatter
    //                 .get(section[0])
    //                 .and_then(|ch| ch.get_sub_chapter(&section[1..]))
    //         },
    //     }
    // }

    // /// Returns a mutable reference to the metadata for modification
    // pub fn mut_metadata(&mut self) -> &mut BookMetadata {
    //     &mut self.metadata
    // }

    // // Returns a reference to the metadata
    // pub fn metadata(&self) -> &BookMetadata {
    //     &self.metadata
    // }
}

