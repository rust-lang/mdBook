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
//! use mdbook::loader::Loader;
//! let loader = Loader::new("./src/");
//! let book = loader.load()?;
//! # Ok(())
//! # }
//! # fn main() { run().unwrap() }
//! ```
//!
//! Alternatively, if you are using the `mdbook` crate as a library and
//! only want to read the `SUMMARY.md` file without having to load the
//! entire book from disk, you can use the `parse_summary()` function.
//!
//! ```rust
//! # fn run() -> mdbook::errors::Result<()> {
//! use mdbook::loader::parse_summary;
//! let src = "# Book Summary
//!
//! [Introduction](./index.md)
//! - [First Chapter](./first/index.md)
//!   - [Sub-Section](./first/subsection.md)
//! - [Second Chapter](./second/index.md)
//! ";
//! let summary = parse_summary(src)?;
//! println!("{:#?}", summary);
//! # Ok(())
//! # }
//! # fn main() { run().unwrap() }
//! ```

#![deny(missing_docs)]

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use errors::*;

mod summary;

pub use self::summary::{Summary, Link, SummaryItem, parse_summary, SectionNumber};


/// The object in charge of parsing the source directory into a usable
/// `Book` struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Loader {
    source_directory: PathBuf,
}

impl Loader {
    /// Create a new loader which uses the provided source directory.
    pub fn new<P: AsRef<Path>>(source_directory: P) -> Loader {
        Loader { source_directory: source_directory.as_ref().to_path_buf() }
    }

    /// Parse the summary file and use it to load a book from disk.
    pub fn load(&self) -> Result<()> {
        let summary = self.parse_summary().chain_err(
            || "Couldn't parse `SUMMARY.md`",
        )?;

        unimplemented!()
    }

    /// Parse the `SUMMARY.md` file.
    pub fn parse_summary(&self) -> Result<Summary> {
        let path = self.source_directory.join("SUMMARY.md");

        let mut summary_content = String::new();
        File::open(&path)?.read_to_string(&mut summary_content)?;

        summary::parse_summary(&summary_content)
    }
}
