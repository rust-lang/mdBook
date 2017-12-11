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
//! - Write a new Renderer
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
//! ## Implementing a new Renderer
//!
//! If you want to create a new renderer for mdBook, the only thing you have to
//! do is to implement the [Renderer](renderer/renderer/trait.Renderer.html)
//! trait.
//!
//! And then you can swap in your renderer like this:
//!
//! ```no_run
//! # extern crate mdbook;
//! #
//! # use mdbook::MDBook;
//! # use mdbook::renderer::HtmlHandlebars;
//! #
//! # #[allow(unused_variables)]
//! # fn main() {
//! #   let your_renderer = HtmlHandlebars::new();
//! #
//! let mut book = MDBook::load("my-book").unwrap();
//! book.set_renderer(your_renderer);
//! # }
//! ```
//!
//! If you make a renderer, you get the book constructed in form of
//! `Vec<BookItems>` and you get ! the book config in a `BookConfig` struct.
//!
//! It's your responsability to create the necessary files in the correct
//! directories.
//!
//! ## utils
//!
//! I have regrouped some useful functions in the [utils](utils/index.html)
//! module, like the following function [`utils::fs::create_file(path:
//! &Path)`](utils/fs/fn.create_file.html).
//!
//! This function creates a file and returns it. But before creating the file
//! it checks every directory in the path to see if it exists, and if it does
//! not it will be created.
//!
//! Make sure to take a look at it.
//!
//! [user guide]: https://rust-lang-nursery.github.io/mdBook/

#[macro_use]
extern crate error_chain;
extern crate handlebars;
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
extern crate tempdir;
extern crate toml;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod preprocess;
pub mod book;
pub mod config;
pub mod renderer;
pub mod theme;
pub mod utils;

pub use book::MDBook;
pub use book::BookItem;
pub use renderer::Renderer;

/// The error types used through out this crate.
pub mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            HandlebarsRender(::handlebars::RenderError);
            HandlebarsTemplate(Box<::handlebars::TemplateError>);
            Utf8(::std::string::FromUtf8Error);
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
        }
    }

    // Box to halve the size of Error
    impl From<::handlebars::TemplateError> for Error {
        fn from(e: ::handlebars::TemplateError) -> Error {
            From::from(Box::new(e))
        }
    }
}
