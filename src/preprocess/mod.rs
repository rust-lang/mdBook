//! Book preprocessing.

pub use self::links::LinkPreprocessor;

mod links;

use book::Book;
use config::Config;
use errors::*;

use std::path::PathBuf;

/// Extra information for a `Preprocessor` to give them more context when 
/// processing a book.
pub struct PreprocessorContext {
    /// The location of the book directory on disk.
    pub root: PathBuf,
    /// The book configuration (`book.toml`).
    pub config: Config,
}

impl PreprocessorContext {
    /// Create a new `PreprocessorContext`.
    pub(crate) fn new(root: PathBuf, config: Config) -> Self {
        PreprocessorContext { root, config }
    }
}

/// An operation which is run immediately after loading a book into memory and 
/// before it gets rendered.
pub trait Preprocessor {
    /// Get the `Preprocessor`'s name.
    fn name(&self) -> &str;

    /// Run this `Preprocessor`, allowing it to update the book before it is
    /// given to a renderer.
    fn run(&self, ctx: &PreprocessorContext, book: &mut Book) -> Result<()>;
}