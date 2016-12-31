pub use self::html_handlebars::HtmlHandlebars;

mod html_handlebars;

use book::MDBook;
use std::error::Error;
use std::path::PathBuf;

pub trait Renderer {

    /// Responsible for creating an `MDBook` struct from path, preparing the
    /// project and calling `render()`, doing what is necessary for the
    /// particular output format.
    ///
    /// This involves parsing config options from `book.toml` and parsing the
    /// `SUMMARY.md` of each translation to a nested `Vec<TocItem>`.
    ///
    /// Finally it calls `render()` to process the chapters and static assets.
    fn build(&self, project_root: &PathBuf) -> Result<(), Box<Error>>;

    /// Responsible for rendering the chapters and copying static assets.
    fn render(&self, book_project: &MDBook) -> Result<(), Box<Error>>;

}
