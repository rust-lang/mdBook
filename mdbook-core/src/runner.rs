#![allow(missing_docs, unused_variables)]

use std::path::{Path, PathBuf};

use config::Config;
use loader::Loader;
use errors::*;

/// The object in charge of coordinating all the individual components necessary
/// to turn your source code into a rendered book.
///
/// It will:
///
/// - Defers to the DirectoryManager to load the `Book` and config from disk
/// - set up the rendering pipeline so the Renderer can transform source text
///   into the final product
/// - Make sure each Plugin is called so they can do their pre/post-processing
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Runner {
    loader: Loader,
}

impl Runner {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Runner> {
        let loader = Loader::new(root).chain_err(|| "Couldn't create the loader")?;
        Ok(Runner { loader })
    }

    /// Initialize a new project directory.
    pub fn init<P: AsRef<Path>>(root: P) -> Result<Self> {
        unimplemented!()
    }

    /// Build the book.
    pub fn build(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Watch the project and rebuild on change.
    pub fn watch(&mut self) {
        unimplemented!()
    }

    /// Serve a HTML version of the book locally, rebuilding on change.
    pub fn serve(&mut self) {
        unimplemented!()
    }
}
