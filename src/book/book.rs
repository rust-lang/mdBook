use book::metadata::BookMetadata;
use book::chapter::Chapter;


/// The `Book` struct contains the metadata and chapters for one language of the book.
/// Multiple `Book` structs are combined in the `MDBook` struct to support multi-language books.
#[derive(Debug, Clone)]
pub struct Book {
    metadata: BookMetadata,

    preface: Vec<Chapter>,
    chapters: Vec<Chapter>,
    appendix: Vec<Chapter>,
}

impl Book {
    /// Creates a new book with the given title, chapters are added with the `add_chapter` method
    pub fn new(title: &str) -> Self {
        Book {
            metadata: BookMetadata::new(title),

            preface: Vec::new(),
            chapters: Vec::new(),
            appendix: Vec::new(),
        }
    }

    /// Adds a new chapter at the end of the book
    pub fn add_chapter(&mut self, chapter: Chapter) -> &mut Self {
        self.chapters.push(chapter);
        self
    }

    /// Adds a new preface chapter to the book, the preface chapters are in the order they were added
    pub fn add_preface_chapter(&mut self, chapter: Chapter) -> &mut Self {
        self.preface.push(chapter);
        self
    }

    /// Adds a new appendix chapter to the book, they are in de order they are added
    pub fn add_appendix_chapter(&mut self, chapter: Chapter) -> &mut Self {
        self.appendix.push(chapter);
        self
    }


    /// This method takes a slice `&[x, y, z]` as parameter and returns the corresponding chapter.
    /// For example, to retrieve chapter 2.3 we would use:
    /// ```
    /// #extern crate mdbook;
    /// #use mdbook::book::Book;
    /// #fn main() {
    /// #let book = Book::new("Test");
    /// let chapter_2_3 = book.get_chapter(&[2, 3]);
    /// #}
    /// ```
    pub fn get_chapter(&self, section: &[usize]) -> Option<&Chapter> {
        match section.len() {
            0 => None,
            1 => self.chapters.get(section[0]),
            _ => {
                self.chapters
                    .get(section[0])
                    .and_then(|ch| ch.get_sub_chapter(&section[1..]))
            },
        }
    }

    /// Returns a mutable reference to the metadata for modification
    pub fn mut_metadata(&mut self) -> &mut BookMetadata {
        &mut self.metadata
    }

    // Returns a reference to the metadata
    pub fn metadata(&self) -> &BookMetadata {
        &self.metadata
    }
}
