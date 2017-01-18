//! # mdBook
//!
//! **mdBook** is similar to Gitbook but implemented in Rust.
//! It offers a command line interface, but can also be used as a regular crate.
//!
//! This is the API doc, but you can find a [less "low-level" documentation here](../index.html) that
//! contains information about the command line tool, format, structure etc.
//! It is also rendered with mdBook to showcase the features and default html template.
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
//! Building a book by the path to its directory:
//!
//! ```ignore
//! extern crate mdbook;
//!
//! use mdbook::MDBook;
//! use mdbook::renderer::HtmlHandlebars;
//! use std::path::PathBuf;
//!
//! fn main() {
//!     let path = PathBuf::from("my-book");  // Path to the book project's root
//!     let renderer = HtmlHandlebars::new();
//!     try!(renderer.build(&path));          // Build the book
//! }
//! ```
//!
//! Or, preparing an `MDBook` struct step-by-step and passing it to a renderer:
//!
//! ```ignore
//! extern crate mdbook;
//!
//! use mdbook::MDBook;
//! use mdbook::renderer::HtmlHandlebars;
//! use std::path::PathBuf;
//!
//! fn main() {
//!     let path = PathBuf::from("my-book");        // Path to the book project's root
//!     let mut book_project = MDBook::new(&path);
//!     book_project.read_config();                 // Parse book.toml file for configuration
//!     book_project.parse_books();                 // Parse SUMMARY.md, build TOC, parse chapters
//!     book_project.link_translations();           // Identity links between translations
//!
//!     let renderer = HtmlHandlebars::new();
//!     try!(renderer.render(&book_project));       // Render the book
//! }
//! ```
//!
//! ## Implementing a new Renderer
//!
//! If you want to create a new renderer for mdBook, implement the [Renderer
//! trait](renderer/renderer/trait.Renderer.html), which is composed of two
//! functions, `.build()` and `.render()`.
//! ```

#[macro_use]
extern crate serde_derive;

extern crate includedir;
extern crate phf;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

extern crate serde;
extern crate serde_json;
extern crate handlebars;
extern crate pulldown_cmark;
extern crate regex;
extern crate glob;

#[macro_use] extern crate log;
pub mod book;
mod parse;
pub mod renderer;
pub mod utils;
pub mod tests;

pub use book::MDBook;
