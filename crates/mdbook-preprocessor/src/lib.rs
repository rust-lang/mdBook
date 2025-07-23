//! Library to assist implementing an mdbook preprocessor.
//!
//! This library is used to implement a
//! [preprocessor](https://rust-lang.github.io/mdBook/for_developers/preprocessors.html)
//! for [mdBook](https://rust-lang.github.io/mdBook/). See the linked chapter
//! for more information on how to implement a preprocessor.

use anyhow::Context;
use mdbook_core::book::Book;
use mdbook_core::config::Config;
use mdbook_core::errors::Result;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

pub use mdbook_core::MDBOOK_VERSION;
pub use mdbook_core::book;
pub use mdbook_core::config;
pub use mdbook_core::errors;

/// An operation which is run immediately after loading a book into memory and
/// before it gets rendered.
///
/// Types that implement the `Preprocessor` trait can be used with
/// [`MDBook::with_preprocessor`] to programmatically add preprocessors.
///
/// [`MDBook::with_preprocessor`]: https://docs.rs/mdbook-driver/latest/mdbook_driver/struct.MDBook.html#method.with_preprocessor
pub trait Preprocessor {
    /// Get the `Preprocessor`'s name.
    fn name(&self) -> &str;

    /// Run this `Preprocessor`, allowing it to update the book before it is
    /// given to a renderer.
    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book>;

    /// A hint to `MDBook` whether this preprocessor is compatible with a
    /// particular renderer.
    ///
    /// By default, always returns `true`.
    fn supports_renderer(&self, _renderer: &str) -> bool {
        true
    }
}

/// Extra information for a `Preprocessor` to give them more context when
/// processing a book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreprocessorContext {
    /// The location of the book directory on disk.
    pub root: PathBuf,
    /// The book configuration (`book.toml`).
    pub config: Config,
    /// The `Renderer` this preprocessor is being used with.
    pub renderer: String,
    /// The calling `mdbook` version.
    pub mdbook_version: String,
    /// Internal mapping of chapter titles.
    ///
    /// This is used internally by mdbook to compute custom chapter titles.
    /// This should not be used outside of mdbook's internals.
    #[serde(skip)]
    pub chapter_titles: RefCell<HashMap<PathBuf, String>>,
    #[serde(skip)]
    __non_exhaustive: (),
}

impl PreprocessorContext {
    /// Create a new `PreprocessorContext`.
    pub fn new(root: PathBuf, config: Config, renderer: String) -> Self {
        PreprocessorContext {
            root,
            config,
            renderer,
            mdbook_version: crate::MDBOOK_VERSION.to_string(),
            chapter_titles: RefCell::new(HashMap::new()),
            __non_exhaustive: (),
        }
    }
}

/// Parses the input given to a preprocessor.
pub fn parse_input<R: Read>(reader: R) -> Result<(PreprocessorContext, Book)> {
    serde_json::from_reader(reader).with_context(|| "Unable to parse the input")
}
