//! Book preprocessing.

pub use self::cmd::CmdPreprocessor;
pub use self::index::IndexPreprocessor;
pub use self::links::LinkPreprocessor;

mod cmd;
mod index;
mod links;

use crate::book::{Book, Chapter};
use crate::config::Config;
use crate::errors::*;

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{
    fmt::{Debug},
};

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
    #[serde(skip)]
    pub(crate) chapter_titles: RefCell<HashMap<PathBuf, String>>,
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

/// An operation which is run immediately after loading a book into memory and
/// before it gets rendered.
pub trait Preprocessor: PreprocessorClone {
    /// Get the `Preprocessor`'s name.
    fn name(&self) -> &str;

    /// Run this `Preprocessor`, allowing it to update the book before it is
    /// given to a renderer.
    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book>;

    /// Pre-Process only one mutable chapter using context and supplied pre-processor
    fn preprocess_chapter(&self, ctx: &PreprocessorContext, chapter: &mut Chapter) -> Result<()> {
        println!("preprocess {} by ctx = {}", chapter.name, ctx.renderer);
        Ok(())
    }

    /// A hint to `MDBook` whether this preprocessor is compatible with a
    /// particular renderer.
    ///
    /// By default, always returns `true`.
    fn supports_renderer(&self, _renderer: &str) -> bool {
        true
    }
}

/// That is the code to have ability to clone vec[Preprocessor]
/// We use for cloning vector of preprocessors and reuse inside 'mdbook-epub'
pub trait PreprocessorClone {
    /// clone one boxed preprocessor
    fn clone_preprocessor(&self) -> Box<dyn Preprocessor>;
}

impl<T: 'static + Preprocessor + Clone> PreprocessorClone for T {
    fn clone_preprocessor(&self) -> Box<dyn Preprocessor> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Preprocessor> {
    fn clone(&self) -> Box<dyn Preprocessor> {
        self.clone_preprocessor()
    }
}

