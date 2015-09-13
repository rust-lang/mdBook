use std::path::Path;
use std::fs::File;
use std::io::Read;

use utils::{PathExt};

pub static INDEX: &'static [u8] = include_bytes!("index.hbs");
pub static CSS: &'static [u8] = include_bytes!("book.css");
pub static JS: &'static [u8] = include_bytes!("book.js");
pub static HIGHLIGHT_JS: &'static [u8] = include_bytes!("highlight.js");
pub static HIGHLIGHT_CSS: &'static [u8] = include_bytes!("highlight.css");
pub static JQUERY: &'static [u8] = include_bytes!("jquery-2.1.4.min.js");

/// The `Theme` struct should be used instead of the static variables because the `new()` method
/// will look if the user has a theme directory in his source folder and use the users theme instead
/// of the default.
///
/// You should exceptionnaly use the static variables only if you need the default theme even if the
/// user has specified another theme.
pub struct Theme {
    pub index: Vec<u8>,
    pub css: Vec<u8>,
    pub js: Vec<u8>,
    pub highlight_css: Vec<u8>,
    pub highlight_js: Vec<u8>,
    pub jquery: Vec<u8>,
}

impl Theme {
    pub fn new(src: &Path) -> Self{

        // Default theme
        let mut theme = Theme {
            index: INDEX.to_owned(),
            css: CSS.to_owned(),
            js: JS.to_owned(),
            highlight_css: HIGHLIGHT_CSS.to_owned(),
            highlight_js: HIGHLIGHT_JS.to_owned(),
            jquery: JQUERY.to_owned(),
        };

        // Check if the given path exists
        if !src.exists() || !src.is_dir() {
            return theme
        }

        let src = src.join("theme");
        // If src does exist, check if there is a theme directory in it
        if !src.exists() || !src.is_dir() {
            return theme
        }

        // Check for individual files if they exist

        // index.hbs
        match File::open(&src.join("index.hbs")) {
            Ok(mut f) => {
                theme.index.clear(); // Reset the value, because read_to_string appends...
                match f.read_to_end(&mut theme.index) {
                    _ => {}
                };
            },
            _ => {},
        }

        // book.js
        match File::open(&src.join("book.js")) {
            Ok(mut f) => {
                theme.js.clear();
                match f.read_to_end(&mut theme.js){
                    _ => {}
                }
            },
            _ => {},
        }

        // book.css
        match File::open(&src.join("book.css")) {
            Ok(mut f) => {
                theme.css.clear();
                match f.read_to_end(&mut theme.css) {
                    _ => {}
                }
            },
            _ => {},
        }

        // highlight.js
        match File::open(&src.join("highlight.js")) {
            Ok(mut f) => {
                theme.highlight_js.clear();
                match f.read_to_end(&mut theme.highlight_js) {
                    _ => {}
                }
            },
            _ => {},
        }

        // highlight.css
        match File::open(&src.join("highlight.css")) {
            Ok(mut f) => {
                theme.highlight_css.clear();
                match f.read_to_end(&mut theme.highlight_css) {
                    _ => {}
                }
            },
            _ => {},
        }

        theme
    }
}
