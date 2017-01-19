pub use self::html_handlebars::HtmlHandlebars;

mod html_handlebars;

use book::MDBook;
use std::error::Error;
use std::path::PathBuf;

// TODO refactor dest_base out of the .build() call. It's only here b/c to
// influence the build output with the --dest-dir CLI arg. It is a good
// thing though that .build() encapsulates the steps to prepare the MDBook
// struct for .render(). Maybe give it the CLI args and process them within
// .build().

pub trait Renderer {

    /// When the output format is determined (by a CLI argument for example),
    /// call `.build()` of the selected Renderer implementation.
    ///
    /// Constructs an `MDBook` struct given the path of the book project,
    /// optionally using a custom output folder (such as when given with
    /// `--dest-dir` CLI argument). It prepares the project and calls
    /// `render()`, doing what is necessary for the particular output format.
    ///
    /// This involves parsing config options from `book.toml` and parsing the
    /// `SUMMARY.md` of each translation to a nested `Vec<TocItem>`.
    ///
    /// Finally it calls `render()` to process the chapters and static assets.
    fn build(&self, project_root: &PathBuf, dest_base: &Option<PathBuf>) -> Result<MDBook, Box<Error>>;

    /// Responsible for rendering the chapters and copying static assets.
    fn render(&self, book_project: &MDBook) -> Result<(), Box<Error>>;

}
