//! Theme dependencies for in-browser search. Not included in mdbook when
//! the "search" cargo feature is disabled.

pub static JS: &[u8] = include_bytes!("../../front-end/searcher/searcher.js");
pub static MARK_JS: &[u8] = include_bytes!("../../front-end/searcher/mark.min.js");
pub static ELASTICLUNR_JS: &[u8] = include_bytes!("../../front-end/searcher/elasticlunr.min.js");
