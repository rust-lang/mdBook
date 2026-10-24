//! mdBook HTML renderer.

mod html;
mod html_handlebars;
pub mod theme;
pub(crate) mod utils;

pub use html_handlebars::HtmlHandlebars;
