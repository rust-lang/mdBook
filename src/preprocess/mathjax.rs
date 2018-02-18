//! Preprocessor that converts mathematical expression into MathJax.
//!
//! This preprocessor takes inline expressions wrapped in `$`-pairs and block
//! expressions wrapped in `$$`-pairs and transform them into a valid MathJax
//! expression that does not interfere with the markdown parser.

use errors::Result;

use super::{Preprocessor, PreprocessorContext};
use book::{Book, BookItem};

/// a preprocessor for expanding `$`- and `$$`-pairs into valid MathJax expressions.
pub struct MathJaxPreprocessor;

impl MathJaxPreprocessor {
    /// Create a `MathJaxPreprocessor`.
    pub fn new() -> Self {
        MathJaxPreprocessor
    }
}

impl Preprocessor for MathJaxPreprocessor {
    fn name(&self) -> &str {
        "mathjax"
    }

    fn run(&self, _ctx: &PreprocessorContext, book: &mut Book) -> Result<()> {
        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *section {
                let content = replace_all_mathematics(&chapter.content);
                chapter.content = content;
            }
        });

        Ok(())
    }
}

fn replace_all_mathematics(content: &str) -> String {
    let mut previous_end_index = 0;
    let mut replaced = String::new();

    for _math in find_mathematics(content) {
        unimplemented!();
    }

    replaced.push_str(&content[previous_end_index..]);

    replaced
}

fn find_mathematics(_content: &str) -> MathematicsIterator {
    MathematicsIterator
}

struct MathematicsIterator;

impl Iterator for MathematicsIterator {
    type Item = Mathematics;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

struct Mathematics {
    start_index: usize,
    end_index: usize,
}
