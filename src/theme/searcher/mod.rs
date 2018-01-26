//! Theme dependencies for in-browser search. Not included in mdbook when
//! the "search" cargo feature is disabled.

use std::path::Path;

use theme::load_file_contents;

pub static JS: &'static [u8] = include_bytes!("searcher.js");
pub static MARK_JS: &'static [u8] = include_bytes!("mark.min.js");
pub static ELASTICLUNR_JS: &'static [u8] = include_bytes!("elasticlunr.min.js");

/// The `Searcher` struct should be used instead of the static variables because
/// the `new()` method will look if the user has a searcher directory in their source folder and use
/// the user's searcher instead of the default.
///
/// You should only use the static variables if you need the default searcher even if the
/// user has specified another searcher.
pub struct Searcher {
    pub js: Vec<u8>,
    pub mark_js: Vec<u8>,
    pub elasticlunr_js: Vec<u8>,
}

impl Searcher {
    pub fn new(src: &Path) -> Self {
        let mut search = Searcher {
            js: JS.to_owned(),
            mark_js: MARK_JS.to_owned(),
            elasticlunr_js: ELASTICLUNR_JS.to_owned(),
        };

        // Check if the given path exists
        if !src.exists() || !src.is_dir() {
            return search;
        }

        // Check for individual files if they exist
        {
            let files = vec![(src.join("searcher.js"), &mut search.js),
                             (src.join("mark.min.js"), &mut search.mark_js),
                             (src.join("elasticlunr.min.js"), &mut search.elasticlunr_js)];

            for (filename, dest) in files {
                if !filename.exists() {
                    continue;
                }

                if let Err(e) = load_file_contents(&filename, dest) {
                    warn!("Couldn't load custom file, {}: {}", filename.display(), e);
                }
            }
        }

        search
    }
}
