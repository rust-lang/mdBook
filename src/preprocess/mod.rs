pub mod links;

use book::Book;
use errors::*;


pub trait Preprocessor {
    fn run(&self, book: &mut Book) -> Result<()>;
}