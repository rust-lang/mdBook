//! Preprocessor that converts mathematical expression into MathJax.
//!
//! This preprocessor takes inline expressions wrapped in `$`-pairs and block
//! expressions wrapped in `$$`-pairs and transform them into a valid MathJax
//! expression that does not interfere with the markdown parser.

use errors::Result;

use super::{Preprocessor, PreprocessorContext};
use book::Book;

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

    fn run(&self, _ctx: &PreprocessorContext, _book: &mut Book) -> Result<()> {
        Ok(())
    }
}
