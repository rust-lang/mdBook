//! # mdBook
//!
//! **mdBook** is similar to Gitbook but implemented in Rust.
//! It offers a command line interface, but can also be used as a regular crate.
//!
//! This is the API doc, but you can find a [less "low-level" documentation here](../index.html) that
//! contains information about the command line tool, format, structure etc.
//! It is also rendered with mdBook to showcase the features and default theme.
//!
//! Some reasons why you would want to use the crate (over the cli):
//!
//! - Integrate mdbook in a current project
//! - Extend the capabilities of mdBook
//! - Do some processing or test before building your book
//! - Write a new Renderer
//! - ...
//!
//! ## Example
//!
//! ```no_run
//! extern crate mdbook;
//!
//! use mdbook::MDBook;
//! use std::path::Path;
//!
//! fn main() {
//!    let mut book =  MDBook::new(Path::new("book-test")) // ERROR directory does not exist...
//!                     .set_src(Path::new("source"))
//!                     .set_dest(Path::new("output"))
//!                     .read_config(); // Reads book.json file for settings
//!
//!     book.init().unwrap();
//!     book.build().unwrap();
//! }
//! ```
//!
//! ## Implementing a new Renderer
//!
//! If you want to create a new renderer for mdBook, the only thing you have to do is to implement
//! the [Renderer trait](renderer/renderer/trait.Renderer.html)
//!
//! And then you can swap in your renderer like this:
//!
//! ```no_run
//! # extern crate mdbook;
//! #
//! # use mdbook::MDBook;
//! # use mdbook::renderer::HtmlHandlebars;
//! # use std::path::Path;
//! #
//! # fn main() {
//! #   let your_renderer = HtmlHandlebars::new();
//! #
//!     let book =  MDBook::new(Path::new("my-book")).set_renderer(Box::new(your_renderer));
//! # }
//! ```
//! If you make a renderer, you get the book constructed in form of `Vec<BookItems>` and you get
//! the book config in a `BookConfig` struct.
//!
//! It's your responsability to create the necessary files in the correct directories.


#[macro_use]
pub mod macros;
pub mod book;
mod parse;
pub mod renderer;
pub mod theme;
pub mod utils;

pub use book::MDBook;
pub use book::BookItem;
pub use book::BookConfig;
pub use renderer::Renderer;
