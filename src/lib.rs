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
//! ```
//! extern crate mdbook;
//!
//! use mdbook::MDBook;
//!
//! fn main() {
//!    let book =  MDBook::new("my-book")
//!                     .set_src("source_dir")
//!                     .set_dest("output_dir")
//!                     .read_config() // Reads book.json file for settings
//!
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
//! ```ignore
//! let book =  MDBook::new("my-book").set_renderer(your_renderer)
//! ```


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
