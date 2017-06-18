//! Traits which must be implemented by plugins and renderers.

use std::path::Path;

use book::Book;
use config::Config;
use errors::*;

/// The trait for rendering a `Book`.
pub trait Renderer {
    /// Get the Renderer's name.
    fn name(&self) -> &str;

    /// Render the book to disk in the specified directory.
    fn render(&mut self, book: &Book, config: &Config, output_directory: &Path);
}

/// A plugin for doing pre/post processing.
pub trait Plugin {
    // TODO: How can a plugin apply renderer-specific operations?
    // e.g. check for dead/broken links in the HTML renderer

    /// A function which is run on the book's raw content immediately after 
    /// being loaded from disk.
    ///
    /// This allows plugin creators to do any special preprocessing before it
    /// reaches the markdown parser (e.g. MathJax substitution). The plugin may
    /// or may not decide to make changes.
    fn preprocess_book(&mut self, book: &mut Book) -> Result<()> {
        Ok(())
    }

    /// The plugin function called after `mdbook` has loaded the book into
    /// memory and just before the renderer writes it to disk.
    ///
    /// This is typically when you would go through and update links or add
    /// in a TOC. You'll typically want to use the `book::Visitor` trait to make
    /// this easier.
    fn postprocess_book(&mut self, _book: &mut Book) -> Result<()> {
        Ok(())
    }
}
