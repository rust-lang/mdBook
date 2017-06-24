pub use self::html_handlebars::HtmlHandlebars;

mod html_handlebars;

use errors::*;

pub trait Renderer {
    fn render(&self, book: &::book::MDBook) -> Result<()>;
}
