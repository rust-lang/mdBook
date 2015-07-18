use book::{BookItem, BookItems};
use book::BookConfig;

use std::error::Error;

pub trait Renderer {
    fn render(&self, book: BookItems, config: &BookConfig) ->  Result<(), Box<Error>>;
}
