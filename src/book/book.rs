use book::metadata::BookMetadata;
use book::chapter::Chapter;


/// The `Book` struct contains the metadata and chapters for one language of the book.
/// Multiple `Book` structs are combined in the `MDBook` struct to support multi-language books.
#[derive(Debug, Clone)]
pub struct Book {
    metadata: BookMetadata,

    frontmatter: Vec<Chapter>,
    mainmatter: Vec<Chapter>,
    backmatter: Vec<Chapter>,
}

impl Book {
    /// Creates a new book with the given title, chapters are added with the
    /// `add_frontmatter_chapter`, `add_mainmatter_chapter`,
    /// `add_backmatter_chapter` methods
    pub fn new(title: &str) -> Self {
        Book {
            metadata: BookMetadata::new(title),

            frontmatter: Vec::new(),
            mainmatter: Vec::new(),
            backmatter: Vec::new(),
        }
    }

    /// Adds a new mainmatter chapter
    pub fn add_mainmatter_chapter(&mut self, chapter: Chapter) -> &mut Self {
        self.mainmatter.push(chapter);
        self
    }

    /// Adds a new frontmatter chapter
    pub fn add_frontmatter_chapter(&mut self, chapter: Chapter) -> &mut Self {
        self.frontmatter.push(chapter);
        self
    }

    /// Adds a new backmatter chapter
    pub fn add_backmatter_chapter(&mut self, chapter: Chapter) -> &mut Self {
        self.backmatter.push(chapter);
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
            1 => self.mainmatter.get(section[0]),
            _ => {
                self.mainmatter
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
