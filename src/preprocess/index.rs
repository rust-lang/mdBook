use errors::*;

use super::{Preprocessor, PreprocessorContext};
use book::{Book, BookItem};

/// A preprocessor for converting file name `README.md` to `index.md` since
/// `README.md` is the de facto index file in a markdown-based documentation.
pub struct IndexPreprocessor;

impl IndexPreprocessor {
    /// Create a new `IndexPreprocessor`.
    pub fn new() -> Self {
        IndexPreprocessor
    }
}

impl Preprocessor for IndexPreprocessor {
    fn name(&self) -> &str {
        "index"
    }

    fn run(&self, _ctx: &PreprocessorContext, book: &mut Book) -> Result<()> {
        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut ch) = *section {
                if ch.path.file_name().unwrap_or_default() == "README.md" {
                    ch.path.set_file_name("index.md");
                }
            }
        });

        Ok(())
    }
}

