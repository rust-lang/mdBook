pub use self::links::LinkPreprocessor;

mod links;

use book::Book;
use config::Config;
use errors::*;

use std::path::PathBuf;

pub struct PreprocessorContext {
    pub root: PathBuf,
    pub config: Config,
}

impl PreprocessorContext {
    pub fn new(root: PathBuf, config: Config) -> Self {
        PreprocessorContext { root, config }
    }
}

pub trait Preprocessor {
    fn name(&self) -> &str;
    fn run(&self, ctx: &PreprocessorContext, book: &mut Book) -> Result<()>;
}