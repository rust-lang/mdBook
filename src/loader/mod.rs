//! Functionality for loading the internal book representation from disk.
//!
//! The typical use case is to create a `Loader` pointing at the correct
//! source directory then call the `load()` method. Internally this will
//! search for the `SUMMARY.md` file, parse it, then use the parsed
//! `Summary` to construct an in-memory representation of the entire book.
//!
//! # Examples
//!
//! ```rust,no_run
//! # fn run() -> mdbook::errors::Result<()> {
//! use mdbook::loader::load_book;
//! let book = load_book("./src/")?;
//! # Ok(())
//! # }
//! # fn main() { run().unwrap() }
//! ```

#![deny(missing_docs)]

use std::path::Path;
use std::fs::File;
use std::io::Read;
use errors::*;

mod summary;
mod book;

pub use self::book::{Book, BookItems, BookItem, Chapter};
pub use self::summary::SectionNumber;

use self::book::load_book_from_disk;
use self::summary::parse_summary;

/// Load a book into memory from its `src/` directory.
pub fn load_book<P: AsRef<Path>>(src_dir: P) -> Result<Book> {
    let src_dir = src_dir.as_ref();
    let summary_md = src_dir.join("SUMMARY.md");

    let mut summary_content = String::new();
    File::open(summary_md)
        .chain_err(|| "Couldn't open SUMMARY.md")?
        .read_to_string(&mut summary_content)?;

    let summary = parse_summary(&summary_content).chain_err(
        || "Summary parsing failed",
    )?;

    load_book_from_disk(&summary, src_dir)
}
