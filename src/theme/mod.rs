#![allow(missing_docs)]

pub mod playpen_editor;

#[cfg(feature = "search")]
pub mod searcher;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use errors::*;

pub static INDEX: &'static [u8] = include_bytes!("index.hbs");
pub static HEADER: &'static [u8] = include_bytes!("header.hbs");
pub static CHROME_CSS: &'static [u8] = include_bytes!("css/chrome.css");
pub static GENERAL_CSS: &'static [u8] = include_bytes!("css/general.css");
pub static PRINT_CSS: &'static [u8] = include_bytes!("css/print.css");
pub static VARIABLES_CSS: &'static [u8] = include_bytes!("css/variables.css");
pub static FAVICON: &'static [u8] = include_bytes!("favicon.png");
pub static JS: &'static [u8] = include_bytes!("book.js");
pub static HIGHLIGHT_JS: &'static [u8] = include_bytes!("highlight.js");
pub static TOMORROW_NIGHT_CSS: &'static [u8] = include_bytes!("tomorrow-night.css");
pub static HIGHLIGHT_CSS: &'static [u8] = include_bytes!("highlight.css");
pub static AYU_HIGHLIGHT_CSS: &'static [u8] = include_bytes!("ayu-highlight.css");
pub static CLIPBOARD_JS: &'static [u8] = include_bytes!("clipboard.min.js");
pub static FONT_AWESOME: &'static [u8] = include_bytes!("FontAwesome/css/font-awesome.min.css");
pub static FONT_AWESOME_EOT: &'static [u8] =
    include_bytes!("FontAwesome/fonts/fontawesome-webfont.eot");
pub static FONT_AWESOME_SVG: &'static [u8] =
    include_bytes!("FontAwesome/fonts/fontawesome-webfont.svg");
pub static FONT_AWESOME_TTF: &'static [u8] =
    include_bytes!("FontAwesome/fonts/fontawesome-webfont.ttf");
pub static FONT_AWESOME_WOFF: &'static [u8] =
    include_bytes!("FontAwesome/fonts/fontawesome-webfont.woff");
pub static FONT_AWESOME_WOFF2: &'static [u8] =
    include_bytes!("FontAwesome/fonts/fontawesome-webfont.woff2");
pub static FONT_AWESOME_OTF: &'static [u8] = include_bytes!("FontAwesome/fonts/FontAwesome.otf");

/// The `Theme` struct should be used instead of the static variables because
/// the `new()` method will look if the user has a theme directory in their
/// source folder and use the users theme instead of the default.
///
/// You should only ever use the static variables directly if you want to
/// override the user's theme with the defaults.
#[derive(Debug, PartialEq)]
pub struct Theme {
    pub index: Vec<u8>,
    pub header: Vec<u8>,
    pub chrome_css: Vec<u8>,
    pub general_css: Vec<u8>,
    pub print_css: Vec<u8>,
    pub variables_css: Vec<u8>,
    pub favicon: Vec<u8>,
    pub js: Vec<u8>,
    pub highlight_css: Vec<u8>,
    pub tomorrow_night_css: Vec<u8>,
    pub ayu_highlight_css: Vec<u8>,
    pub highlight_js: Vec<u8>,
    pub clipboard_js: Vec<u8>,
}

impl Theme {
    /// Creates a `Theme` from the given `theme_dir`.
    /// If a file is found in the theme dir, it will override the default version.
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
                (theme_dir.join("css/chrome.css"), &mut theme.chrome_css),
                (theme_dir.join("css/general.css"), &mut theme.general_css),
                (theme_dir.join("css/print.css"), &mut theme.print_css),
                (
                    theme_dir.join("css/variables.css"),
                    &mut theme.variables_css,
                ),
                (theme_dir.join("favicon.png"), &mut theme.favicon),
                (theme_dir.join("highlight.js"), &mut theme.highlight_js),
                (theme_dir.join("clipboard.min.js"), &mut theme.clipboard_js),
                (theme_dir.join("highlight.css"), &mut theme.highlight_css),
                (
                    theme_dir.join("tomorrow-night.css"),
                    &mut theme.tomorrow_night_css,
                ),
                (
                    theme_dir.join("ayu-highlight.css"),
                    &mut theme.ayu_highlight_css,
                ),
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
            chrome_css: CHROME_CSS.to_owned(),
            general_css: GENERAL_CSS.to_owned(),
            print_css: PRINT_CSS.to_owned(),
            variables_css: VARIABLES_CSS.to_owned(),
            favicon: FAVICON.to_owned(),
            js: JS.to_owned(),
            highlight_css: HIGHLIGHT_CSS.to_owned(),
            tomorrow_night_css: TOMORROW_NIGHT_CSS.to_owned(),
            ayu_highlight_css: AYU_HIGHLIGHT_CSS.to_owned(),
            highlight_js: HIGHLIGHT_JS.to_owned(),
            clipboard_js: CLIPBOARD_JS.to_owned(),
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
    use std::fs;
    use std::path::PathBuf;
    use tempfile::Builder as TempFileBuilder;

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
        let files = [
            "index.hbs",
            "header.hbs",
            "favicon.png",
            "css/chrome.css",
            "css/general.css",
            "css/print.css",
            "css/variables.css",
            "book.js",
            "highlight.js",
            "tomorrow-night.css",
            "highlight.css",
            "ayu-highlight.css",
            "clipboard.min.js",
        ];

        let temp = TempFileBuilder::new().prefix("mdbook-").tempdir().unwrap();
        fs::create_dir(temp.path().join("css")).unwrap();

        // "touch" all of the special files so we have empty copies
        for file in &files {
            File::create(&temp.path().join(file)).unwrap();
        }

        let got = Theme::new(temp.path());

        let empty = Theme {
            index: Vec::new(),
            header: Vec::new(),
            chrome_css: Vec::new(),
            general_css: Vec::new(),
            print_css: Vec::new(),
            variables_css: Vec::new(),
            favicon: Vec::new(),
            js: Vec::new(),
            highlight_css: Vec::new(),
            tomorrow_night_css: Vec::new(),
            ayu_highlight_css: Vec::new(),
            highlight_js: Vec::new(),
            clipboard_js: Vec::new(),
        };

        assert_eq!(got, empty);
    }
}
