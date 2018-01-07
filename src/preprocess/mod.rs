pub use self::links::LinkPreprocessor;

mod links;

use book::Book;
use errors::*;

use std::path::PathBuf;

pub struct PreprocessorContext {
    pub src_dir: PathBuf
}

pub trait Preprocessor {
    fn run(&self, ctx: &PreprocessorContext, book: &mut Book) -> Result<()>;
}