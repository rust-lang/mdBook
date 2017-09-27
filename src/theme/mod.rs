pub mod playpen_editor;

use std::path::Path;
use std::fs::File;
use std::io::Read;

use errors::*;

pub static INDEX: &'static [u8] = include_bytes!("index.hbs");
pub static HEADER: &'static [u8] = include_bytes!("header.hbs");
pub static CSS: &'static [u8] = include_bytes!("book.css");
pub static FAVICON: &'static [u8] = include_bytes!("favicon.png");
pub static JS: &'static [u8] = include_bytes!("book.js");
pub static HIGHLIGHT_JS: &'static [u8] = include_bytes!("highlight.js");
pub static TOMORROW_NIGHT_CSS: &'static [u8] = include_bytes!("tomorrow-night.css");
pub static HIGHLIGHT_CSS: &'static [u8] = include_bytes!("highlight.css");
pub static AYU_HIGHLIGHT_CSS: &'static [u8] = include_bytes!("ayu-highlight.css");
pub static JQUERY: &'static [u8] = include_bytes!("jquery.js");
pub static CLIPBOARD_JS: &'static [u8] = include_bytes!("clipboard.min.js");
pub static STORE_JS: &'static [u8] = include_bytes!("store.js");
pub static FONT_AWESOME: &'static [u8] = include_bytes!("_FontAwesome/css/font-awesome.min.css");
pub static FONT_AWESOME_EOT: &'static [u8] =
    include_bytes!("_FontAwesome/fonts/fontawesome-webfont.eot");
pub static FONT_AWESOME_SVG: &'static [u8] =
    include_bytes!("_FontAwesome/fonts/fontawesome-webfont.svg");
pub static FONT_AWESOME_TTF: &'static [u8] =
    include_bytes!("_FontAwesome/fonts/fontawesome-webfont.ttf");
pub static FONT_AWESOME_WOFF: &'static [u8] =
    include_bytes!("_FontAwesome/fonts/fontawesome-webfont.woff");
pub static FONT_AWESOME_WOFF2: &'static [u8] =
    include_bytes!("_FontAwesome/fonts/fontawesome-webfont.woff2");
pub static FONT_AWESOME_OTF: &'static [u8] = include_bytes!("_FontAwesome/fonts/FontAwesome.otf");


/// The `Theme` struct should be used instead of the static variables because
/// the `new()` method will look if the user has a theme directory in his
/// source folder and use the users theme instead of the default.
///
/// You should only ever use the static variables directly if you want to
/// override the user's theme with the defaults.
#[derive(Debug, PartialEq)]
pub struct Theme {
    pub index: Vec<u8>,
    pub header: Vec<u8>,
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
    pub fn new<P: AsRef<Path>>(theme_dir: P) -> Self {
        let theme_dir = theme_dir.as_ref();
        let mut theme = Theme::default();

        // If the theme directory doesn't exist there's no point continuing...
        if !theme_dir.exists() || !theme_dir.is_dir() {
            return theme;
        }

        // Check for individual files, if they exist copy them across
        {
            let files = vec![
                (theme_dir.join("index.hbs"), &mut theme.index),
                (theme_dir.join("header.hbs"), &mut theme.header),
                (theme_dir.join("book.js"), &mut theme.js),
                (theme_dir.join("book.css"), &mut theme.css),
                (theme_dir.join("favicon.png"), &mut theme.favicon),
                (theme_dir.join("highlight.js"), &mut theme.highlight_js),
                (theme_dir.join("clipboard.min.js"), &mut theme.clipboard_js),
                (theme_dir.join("store.js"), &mut theme.store_js),
                (theme_dir.join("highlight.css"), &mut theme.highlight_css),
                (theme_dir.join("tomorrow-night.css"), &mut theme.tomorrow_night_css),
                (theme_dir.join("ayu-highlight.css"), &mut theme.ayu_highlight_css),
                (theme_dir.join("jquery.js"), &mut theme.jquery),
            ];

            for (filename, dest) in files {
                if !filename.exists() {
                    continue;
                }

                if let Err(e) = load_file_contents(&filename, dest) {
                    warn!("Couldn't load custom file, {}: {}", filename.display(), e);
                }
            }
        }

        theme
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme {
            index: INDEX.to_owned(),
            header: HEADER.to_owned(),
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
        }
    }
}

/// Checks if a file exists, if so, the destination buffer will be filled with
/// its contents.
fn load_file_contents<P: AsRef<Path>>(filename: P, dest: &mut Vec<u8>) -> Result<()> {
    let filename = filename.as_ref();

    let mut buffer = Vec::new();
    File::open(filename)?.read_to_end(&mut buffer)?;

    // We needed the buffer so we'd only overwrite the existing content if we
    // could successfully load the file into memory.
    dest.clear();
    dest.append(&mut buffer);

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;
    use std::path::PathBuf;

    #[test]
    fn theme_uses_defaults_with_nonexistent_src_dir() {
        let non_existent = PathBuf::from("/non/existent/directory/");
        assert!(!non_existent.exists());

        let should_be = Theme::default();
        let got = Theme::new(&non_existent);

        assert_eq!(got, should_be);
    }

    #[test]
    fn theme_dir_overrides_defaults() {
        // Get all the non-Rust files in the theme directory
        let special_files = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/theme")
            .read_dir()
            .unwrap()
            .filter_map(|f| f.ok())
            .map(|f| f.path())
            .filter(|p| p.is_file() && !p.ends_with(".rs"));

        let temp = TempDir::new("mdbook").unwrap();

        // "touch" all of the special files so we have empty copies
        for special_file in special_files {
            let filename = temp.path().join(special_file.file_name().unwrap());
            let _ = File::create(&filename);
        }

        let got = Theme::new(temp.path());

        let empty = Theme {
            index: Vec::new(),
            header: Vec::new(),
            css: Vec::new(),
            favicon: Vec::new(),
            js: Vec::new(),
            highlight_css: Vec::new(),
            tomorrow_night_css: Vec::new(),
            ayu_highlight_css: Vec::new(),
            highlight_js: Vec::new(),
            clipboard_js: Vec::new(),
            store_js: Vec::new(),
            jquery: Vec::new(),
        };

        assert_eq!(got, empty);
    }
}
