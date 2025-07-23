//! Library to assist implementing an mdbook renderer.
//!
//! This library is used to implement a
//! [renderer](https://rust-lang.github.io/mdBook/for_developers/backends.html)
//! for [mdBook](https://rust-lang.github.io/mdBook/). See the linked chapter
//! for more information on how to implement a renderer.

use anyhow::Context;
use mdbook_core::book::Book;
use mdbook_core::config::Config;
use mdbook_core::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

pub use mdbook_core::MDBOOK_VERSION;
pub use mdbook_core::book;
pub use mdbook_core::config;
pub use mdbook_core::errors;

/// An mdbook backend.
///
/// Types that implement the `Renderer` trait can be used with
/// [`MDBook::with_renderer`] to programmatically add renderers.
///
/// [`MDBook::with_renderer`]: https://docs.rs/mdbook-driver/latest/mdbook_driver/struct.MDBook.html#method.with_renderer
pub trait Renderer {
    /// The `Renderer`'s name.
    fn name(&self) -> &str;

    /// Invoke the `Renderer`, passing in all the necessary information for
    /// describing a book.
    fn render(&self, ctx: &RenderContext) -> Result<()>;
}

/// The context provided to all renderers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderContext {
    /// Which version of `mdbook` did this come from (as written in `mdbook`'s
    /// `Cargo.toml`). Useful if you know the renderer is only compatible with
    /// certain versions of `mdbook`.
    pub version: String,
    /// The book's root directory.
    pub root: PathBuf,
    /// A loaded representation of the book itself.
    pub book: Book,
    /// The loaded configuration file.
    pub config: Config,
    /// Where the renderer *must* put any build artefacts generated. To allow
    /// renderers to cache intermediate results, this directory is not
    /// guaranteed to be empty or even exist.
    pub destination: PathBuf,
    /// Internal mapping of chapter titles.
    ///
    /// This is used internally by mdbook to compute custom chapter titles.
    /// This should not be used outside of mdbook's internals.
    #[serde(skip)]
    pub chapter_titles: HashMap<PathBuf, String>,
    #[serde(skip)]
    __non_exhaustive: (),
}

impl RenderContext {
    /// Create a new `RenderContext`.
    pub fn new<P, Q>(root: P, book: Book, config: Config, destination: Q) -> RenderContext
    where
        P: Into<PathBuf>,
        Q: Into<PathBuf>,
    {
        RenderContext {
            book,
            config,
            version: crate::MDBOOK_VERSION.to_string(),
            root: root.into(),
            destination: destination.into(),
            chapter_titles: HashMap::new(),
            __non_exhaustive: (),
        }
    }

    /// Get the source directory's (absolute) path on disk.
    pub fn source_dir(&self) -> PathBuf {
        self.root.join(&self.config.book.src)
    }

    /// Load a `RenderContext` from its JSON representation.
    pub fn from_json<R: Read>(reader: R) -> Result<RenderContext> {
        serde_json::from_reader(reader).with_context(|| "Unable to deserialize the `RenderContext`")
    }
}
