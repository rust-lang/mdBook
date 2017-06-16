pub use self::html_handlebars::HtmlHandlebars;

mod html_handlebars;

use std::error::Error;
use std::io::{self, ErrorKind};

pub trait Renderer {
    fn render(&self, book: &::book::MDBook) -> Result<(), Box<Error>>;
}
