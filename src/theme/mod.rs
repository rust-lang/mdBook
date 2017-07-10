use std::path::Path;
use std::fs::File;
use std::io::Read;


pub static INDEX: &'static [u8] = include_bytes!("index.hbs");
pub static CSS: &'static [u8] = include_bytes!("book.css");
pub static FAVICON: &'static [u8] = include_bytes!("favicon.png");
pub static JS: &'static [u8] = include_bytes!("book.js");
pub static HIGHLIGHT_JS: &'static [u8] = include_bytes!("highlight.js");
pub static TOMORROW_NIGHT_CSS: &'static [u8] = include_bytes!("tomorrow-night.css");
pub static HIGHLIGHT_CSS: &'static [u8] = include_bytes!("highlight.css");
pub static AYU_HIGHLIGHT_CSS: &'static [u8] = include_bytes!("ayu-highlight.css");
pub static JQUERY: &'static [u8] = include_bytes!("jquery-2.1.4.min.js");
pub static CLIPBOARD_JS: &'static [u8] = include_bytes!("clipboard.min.js");
pub static STORE_JS: &'static [u8] = include_bytes!("store.js");
pub static FONT_AWESOME: &'static [u8] = include_bytes!("_FontAwesome/css/font-awesome.min.css");
pub static FONT_AWESOME_EOT: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.eot");
pub static FONT_AWESOME_SVG: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.svg");
pub static FONT_AWESOME_TTF: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.ttf");
pub static FONT_AWESOME_WOFF: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.woff");
pub static FONT_AWESOME_WOFF2: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.woff2");
pub static FONT_AWESOME_OTF: &'static [u8] = include_bytes!("_FontAwesome/fonts/FontAwesome.otf");

/// The `Theme` struct should be used instead of the static variables because
/// the `new()` method
/// will look if the user has a theme directory in his source folder and use
/// the users theme instead
/// of the default.
///
/// You should exceptionnaly use the static variables only if you need the
/// default theme even if the
/// user has specified another theme.
pub struct Theme {
    pub index: Vec<u8>,
    pub css: Vec<u8>,
    pub favicon: Vec<u8>,
    pub js: Vec<u8>,
    pub highlight_css: Vec<u8>,
    pub tomorrow_night_css: Vec<u8>,
    pub ayu_highlight_css: Vec<u8>,
    pub highlight_js: Vec<u8>,
    pub clipboard_js: Vec<u8>,
    pub store_js: Vec<u8>,
    pub jquery: Vec<u8>,
}

impl Theme {
    pub fn new(src: &Path) -> Self {

        // Default theme
        let mut theme = Theme {
            index: INDEX.to_owned(),
            css: CSS.to_owned(),
            favicon: FAVICON.to_owned(),
            js: JS.to_owned(),
            highlight_css: HIGHLIGHT_CSS.to_owned(),
            tomorrow_night_css: TOMORROW_NIGHT_CSS.to_owned(),
            ayu_highlight_css: AYU_HIGHLIGHT_CSS.to_owned(),
            highlight_js: HIGHLIGHT_JS.to_owned(),
            clipboard_js: CLIPBOARD_JS.to_owned(),
            store_js: STORE_JS.to_owned(),
            jquery: JQUERY.to_owned(),
        };

        // Check if the given path exists
        if !src.exists() || !src.is_dir() {
            return theme;
        }

        // Check for individual files, if they exist copy them across
        {
            let files = vec![
                (src.join("index.hbs"), &mut theme.index),
                (src.join("book.js"), &mut theme.js),
                (src.join("book.css"), &mut theme.css),
                (src.join("favicon.png"), &mut theme.favicon),
                (src.join("highlight.js"), &mut theme.highlight_js),
                (src.join("clipboard.min.js"), &mut theme.clipboard_js),
                (src.join("store.js"), &mut theme.store_js),
                (src.join("highlight.css"), &mut theme.highlight_css),
                (src.join("tomorrow-night.css"), &mut theme.tomorrow_night_css),
                (src.join("ayu-highlight.css"), &mut theme.ayu_highlight_css),
            ];

            for (filename, dest) in files {
                load_file_contents(filename, dest);
            }
        }

        theme
    }
}

fn load_file_contents<P: AsRef<Path>>(filename: P, dest: &mut Vec<u8>) {
    let filename = filename.as_ref();

    if let Ok(mut f) = File::open(filename) {
        dest.clear();
        if let Err(e) = f.read_to_end(dest) {
            warn!("Couldn't load custom file, {}: {}", filename.display(), e);
        }
    }
}