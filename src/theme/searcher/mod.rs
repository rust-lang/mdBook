//! Theme dependencies for in-browser search. Not included in mdbook when
//! the "search" cargo feature is disabled.

pub static JS: &'static [u8] = include_bytes!("searcher.js");
pub static MARK_JS: &'static [u8] = include_bytes!("mark.min.js");
pub static ELASTICLUNR_JS: &'static [u8] = include_bytes!("elasticlunr.min.js");
