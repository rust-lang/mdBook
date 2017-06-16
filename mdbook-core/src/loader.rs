use std::path::{Path, PathBuf};

use config::{load_config, Config};
use errors::*;

/// Loader is the object in charge of loading the source documents from disk.
///
/// It Will:
///
/// - Initialize a new project
/// - Parse `SUMMARY.md`
/// - Traverse the source directory, looking for markdown files
/// - Turn all of that into a single data structure which is an in-memory
///   representation of the book
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Loader {
    root: PathBuf,
    config: Config,
}

impl Loader {
    /// Create a new `Loader` with `root` as the book's root directory.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Loader> {
        let root = PathBuf::from(root.as_ref());

        let config = load_config(&root)?;
        Ok(Loader {
            root: root,
            config: config,
        })
    }
}
