//! Functionality for loading the internal book representation from disk.

#![deny(missing_docs)]

use std::path::{Path, PathBuf};
use std::error::Error;


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

    /// Parse the `SUMMARY.md` file.
    pub fn parse_summary(&self) -> Result<Summary, Box<Error>> {
        unimplemented!()
    }
}

/// The parsed `SUMMARY.md`, specifying how the book should be laid out.
pub struct Summary;