//! Theme dependencies for in-browser search. Not included in mdbook when
//! the "search" cargo feature is disabled.

use crate::theme::ContentToMinify;

pub static JS: ContentToMinify<'static> = ContentToMinify::JS(include_str!("searcher.js"));
pub static MARK_JS: &[u8] = include_bytes!("mark.min.js");
pub static ELASTICLUNR_JS: &[u8] = include_bytes!("elasticlunr.min.js");
