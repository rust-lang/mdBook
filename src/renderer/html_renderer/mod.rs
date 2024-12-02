#![allow(missing_docs)] // FIXME: Document this

pub use self::hbs_renderer::HtmlRenderer;

mod hbs_renderer;
mod toc;

#[cfg(feature = "search")]
mod search;
