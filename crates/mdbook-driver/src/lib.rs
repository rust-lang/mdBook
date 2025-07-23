//! High-level library for running mdBook.
//!
//! This is the high-level library for running
//! [mdBook](https://rust-lang.github.io/mdBook/). There are several
//! reasons for using the programmatic API (over the CLI):
//!
//! - Integrate mdBook in a current project.
//! - Extend the capabilities of mdBook.
//! - Do some processing or test before building your book.
//! - Accessing the public API to help create a new Renderer.
//!
//! ## Additional crates
//!
//! In addition to `mdbook-driver`, there are several other crates available
//! for using and extending mdBook:
//!
//! - [`mdbook_preprocessor`]: Provides support for implementing preprocessors.
//! - [`mdbook_renderer`]: Provides support for implementing renderers.
//! - [`mdbook_markdown`]: The Markdown renderer.
//! - [`mdbook_summary`]: The `SUMMARY.md` parser.
//! - [`mdbook_html`]: The HTML renderer.
//! - [`mdbook_core`]: An internal library that is used by the other crates
//!   for shared types. Types from this crate are rexported from the other
//!   crates as appropriate.
//!
//! ## Examples
//!
//! If creating a new book from scratch, you'll want to get a [`init::BookBuilder`] via
//! the [`MDBook::init()`] method.
//!
//! ```rust,no_run
//! use mdbook_driver::MDBook;
//! use mdbook_driver::config::Config;
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
//! use mdbook_driver::MDBook;
//!
//! let root_dir = "/path/to/book/root";
//!
//! let mut md = MDBook::load(root_dir)
//!     .expect("Unable to load the book");
//! md.build().expect("Building failed");
//! ```

pub mod builtin_preprocessors;
pub mod builtin_renderers;
pub mod init;
mod load;
mod mdbook;

pub use mdbook::MDBook;
pub use mdbook_core::{book, config, errors};
