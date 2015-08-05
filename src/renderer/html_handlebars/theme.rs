use std::path::Path;
use std::fs::{File, metadata};
use std::io::Read;
use theme;

pub struct Theme {
    pub index: String,
    pub css: Vec<u8>,
    pub js: Vec<u8>,
    pub highlight_css: Vec<u8>,
    pub highlight_js: Vec<u8>,
}

impl Theme {
    pub fn new(src: &Path) -> Self{

        // Default theme
        let mut theme = Theme {
            index: theme::get_index_hbs().to_owned(),
            css: theme::get_css().to_owned(),
            js: theme::get_js().to_owned(),
            highlight_css: theme::get_highlight_css().to_owned(),
            highlight_js: theme::get_highlight_js().to_owned(),
        };

        // Check if the given path exists
        // Hacky way to check if the path exists... Until PathExt moves to stable
        match metadata(&src) {
            Err(_) => return theme,
            Ok(f) => {
                if !f.is_dir() {
                    return theme;
                }
            },
        }

        let src = src.join("theme");
        // If src does exist, check if there is a theme directory in it
        // Hacky way to check if the path exists... Until PathExt moves to stable
        match metadata(&src) {
            Err(_) => return theme,
            Ok(f) => {
                if !f.is_dir() {
                    return theme;
                }
            }
        }

        // Check for individual files if they exist

        // index.hbs
        match File::open(&src.join("index.hbs")) {
            Ok(mut f) => {
                f.read_to_string(&mut theme.index).unwrap();
            },
            _ => {},
        }

        // book.js
        match File::open(&src.join("book.js")) {
            Ok(mut f) => {
                f.read_to_end(&mut theme.js).unwrap();
            },
            _ => {},
        }

        // book.css
        match File::open(&src.join("book.css")) {
            Ok(mut f) => {
                f.read_to_end(&mut theme.css).unwrap();
            },
            _ => {},
        }

        // highlight.js
        match File::open(&src.join("highlight.js")) {
            Ok(mut f) => {
                f.read_to_end(&mut theme.highlight_js).unwrap();
            },
            _ => {},
        }

        // highlight.css
        match File::open(&src.join("highlight.css")) {
            Ok(mut f) => {
                f.read_to_end(&mut theme.highlight_css).unwrap();
            },
            _ => {},
        }

        theme
    }
}
