#![allow(missing_docs, unused_variables)]

use std::path::{Path, PathBuf};

use config::Config;

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
    config: Config,
}

impl Runner {
    pub fn with_config(config: Config) -> Runner {
        Runner { config }
    }
}
