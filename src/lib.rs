//! # mdBook
//!
//! **mdBook** is similar to GitBook but implemented in Rust.
//! It offers a command line interface, but can also be used as a regular crate.
//!
//! This is the API doc, the [user guide] is also available if you want
//! information about the command line tool, format, structure etc. It is also
//! rendered with mdBook to showcase the features and default theme.
//!
//! Some reasons why you would want to use the crate (over the cli):
//!
//! - Integrate mdbook in a current project
//! - Extend the capabilities of mdBook
//! - Do some processing or test before building your book
//! - Accessing the public API to help create a new Renderer
//! - ...
//!
//! # Examples
//!
//! If creating a new book from scratch, you'll want to get a `BookBuilder` via
//! the `MDBook::init()` method.
//!
//! ```rust,no_run
//! use mdbook::MDBook;
//! use mdbook::config::Config;
//!
//! let root_dir = "/path/to/book/root";
//!
//! // create a default config and change a couple things
//! let mut cfg = Config::default();
//! cfg.book.title = Some("My Book".to_string());
//! cfg.book.authors.push("Michael-F-Bryan".to_string());
//!
//! MDBook::init(root_dir)
//!     .create_gitignore(true)
//!     .with_config(cfg)
//!     .build()
//!     .expect("Book generation failed");
//! ```
//!
//! You can also load an existing book and build it.
//!
//! ```rust,no_run
//! use mdbook::MDBook;
//!
//! let root_dir = "/path/to/book/root";
//!
//! let mut md = MDBook::load(root_dir)
//!     .expect("Unable to load the book");
//! md.build().expect("Building failed");
//! ```
//!
//! ## Implementing a new Backend
//!
//! `mdbook` has a fairly flexible mechanism for creating additional backends 
//! for your book. The general idea is you'll add an extra table in the book's
//! `book.toml` which specifies an executable to be invoked by `mdbook`. This
//! executable will then be called during a build, with an in-memory 
//! representation ([`RenderContext`]) of the book being passed to the
//! subprocess via `stdin`. 
//! 
//! The [`RenderContext`] gives the backend access to the contents of 
//! `book.toml` and lets it know which directory all generated artefacts should
//! be placed in. For a much more in-depth explanation, consult the [For
//! Developers] section of the user guide.
//! 
//! To make creating a backend easier, the `mdbook` crate can be imported 
//! directly, making deserializing the `RenderContext` easy and giving you 
//! access to the various methods for working with the [`Config`].
//!
//! [user guide]: https://rust-lang-nursery.github.io/mdBook/
//! [`RenderContext`]: renderer/struct.RenderContext.html
//! [For Developers]: https://rust-lang-nursery.github.io/mdBook/lib/index.html
//! [`Config`]: config/struct.Config.html

#[macro_use]
extern crate error_chain;
extern crate handlebars;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate memchr;
extern crate pulldown_cmark;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate shlex;
extern crate tempdir;
extern crate toml;
extern crate toml_query;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

pub mod preprocess;
pub mod book;
pub mod config;
pub mod renderer;
pub mod theme;
pub mod utils;

pub use book::MDBook;
pub use book::BookItem;
pub use renderer::Renderer;
pub use config::Config;

/// The error types used through out this crate.
pub mod errors {
    use std::path::PathBuf;

    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            HandlebarsRender(::handlebars::RenderError);
            HandlebarsTemplate(Box<::handlebars::TemplateError>);
            Utf8(::std::string::FromUtf8Error);
        }

        links {
            TomlQuery(::toml_query::error::Error, ::toml_query::error::ErrorKind);
        }

        errors {
            Subprocess(message: String, output: ::std::process::Output) {
                description("A subprocess failed")
                display("{}: {}", message, String::from_utf8_lossy(&output.stdout))
            }

            ParseError(line: usize, col: usize, message: String) {
                description("A SUMMARY.md parsing error")
                display("Error at line {}, column {}: {}", line, col, message)
            }

            ReservedFilenameError(filename: PathBuf) {
                description("Reserved Filename")
                display("{} is reserved for internal use", filename.display())
            }
        }
    }

    // Box to halve the size of Error
    impl From<::handlebars::TemplateError> for Error {
        fn from(e: ::handlebars::TemplateError) -> Error {
            From::from(Box::new(e))
        }
    }
}
