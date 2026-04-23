//! mdBook HTML renderer.

mod html;
mod html_handlebars;
pub mod theme;
pub(crate) mod utils;

#[cfg(feature = "frontmatter")]
pub(crate) mod frontmatter;

pub use html_handlebars::HtmlHandlebars;
