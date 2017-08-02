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
//! use mdbook::config::BookConfig;
//! use mdbook::book::Builder;
//!
//! // configure our book
//! let config = BookConfig::new("my-book").with_source("src");
//!
//! // then create a book which uses that configuration
//! let mut book = Builder::new("my-book").with_config(config).build().unwrap();
//!
//! // and finally, render the book as html
//! book.build().unwrap();
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
//! # #![allow(unused_variables)]
//! use mdbook::renderer::HtmlHandlebars;
//! use mdbook::book::Builder;
//!
//! let your_renderer = HtmlHandlebars::new();
//! let book = Builder::new("my-book").set_renderer(Box::new(your_renderer))
//!                                   .build()
//!                                   .unwrap();
//! ```
//! If you make a renderer, you get the book constructed in form of `Vec<BookItems>` and you get
//! the book config in a `BookConfig` struct.
//!
//! It's your responsibility to create the necessary files in the correct
//! directories.
//!
//! ## utils
//!
//! I have regrouped some useful functions in the [utils](utils/index.html) module, like the
//! following function [`utils::fs::create_file(path:
//! &Path)`](utils/fs/fn.create_file.html)
//!
//! This function creates a file and returns it. But before creating the file
//! it checks every directory in the path to see if it exists, and if it does
//! not it will be created.
//!
//! Make sure to take a look at it.

#[macro_use]
extern crate error_chain;
extern crate handlebars;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate pulldown_cmark;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate tempdir;

mod preprocess;
pub mod book;
pub mod config;
pub mod renderer;
pub mod theme;
pub mod utils;
pub mod loader;

pub use book::MDBook;
pub use loader::{Book, BookItem};
pub use renderer::Renderer;

/// The error types used through out this crate.
pub mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            HandlebarsRender(::handlebars::RenderError);
            HandlebarsTemplate(::handlebars::TemplateError);
            Utf8(::std::string::FromUtf8Error);
        }

        errors {
            Subprocess(message: String, output: ::std::process::Output) {
                description("A subprocess failed")
                display("{}: {}", message, String::from_utf8_lossy(&output.stdout))
            }
        }
    }
}
