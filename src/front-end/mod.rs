#![allow(missing_docs)]

pub mod playground_editor;

pub mod fonts;

#[cfg(feature = "search")]
pub mod searcher;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::errors::*;
use log::warn;
pub static INDEX: &[u8] = include_bytes!("templates/index.hbs");
pub static HEAD: &[u8] = include_bytes!("templates/head.hbs");
pub static REDIRECT: &[u8] = include_bytes!("templates/redirect.hbs");
pub static HEADER: &[u8] = include_bytes!("templates/header.hbs");
pub static TOC_JS: &[u8] = include_bytes!("templates/toc.js.hbs");
pub static TOC_HTML: &[u8] = include_bytes!("templates/toc.html.hbs");
pub static CHROME_CSS: ContentToMinify<'static> =
    ContentToMinify::CSS(include_str!("css/chrome.css"));
pub static GENERAL_CSS: ContentToMinify<'static> =
    ContentToMinify::CSS(include_str!("css/general.css"));
pub static PRINT_CSS: ContentToMinify<'static> =
    ContentToMinify::CSS(include_str!("css/print.css"));
pub static VARIABLES_CSS: ContentToMinify<'static> =
    ContentToMinify::CSS(include_str!("css/variables.css"));
pub static FAVICON_PNG: &[u8] = include_bytes!("images/favicon.png");
pub static FAVICON_SVG: &[u8] = include_bytes!("images/favicon.svg");
pub static JS: ContentToMinify<'static> = ContentToMinify::JS(include_str!("js/book.js"));
pub static HIGHLIGHT_JS: &[u8] = include_bytes!("js/highlight.js");
pub static TOMORROW_NIGHT_CSS: ContentToMinify<'static> =
    ContentToMinify::CSS(include_str!("css/tomorrow-night.css"));
pub static HIGHLIGHT_CSS: ContentToMinify<'static> =
    ContentToMinify::CSS(include_str!("css/highlight.css"));
pub static AYU_HIGHLIGHT_CSS: ContentToMinify<'static> =
    ContentToMinify::CSS(include_str!("css/ayu-highlight.css"));
pub static CLIPBOARD_JS: &[u8] = include_bytes!("js/clipboard.min.js");
pub static FONT_AWESOME: &[u8] = include_bytes!("css/font-awesome.min.css");
pub static FONT_AWESOME_EOT: &[u8] = include_bytes!("fonts/fontawesome-webfont.eot");
pub static FONT_AWESOME_SVG: &[u8] = include_bytes!("fonts/fontawesome-webfont.svg");
pub static FONT_AWESOME_TTF: &[u8] = include_bytes!("fonts/fontawesome-webfont.ttf");
pub static FONT_AWESOME_WOFF: &[u8] = include_bytes!("fonts/fontawesome-webfont.woff");
pub static FONT_AWESOME_WOFF2: &[u8] = include_bytes!("fonts/fontawesome-webfont.woff2");
pub static FONT_AWESOME_OTF: &[u8] = include_bytes!("fonts/FontAwesome.otf");

#[derive(Clone, Copy)]
pub enum ContentToMinify<'a> {
    CSS(&'a str),
    JS(&'a str),
}

impl<'a> ContentToMinify<'a> {
    /// If `minification` is false, it simply returns the inner data converted into a `Vec`.
    pub fn minified(self, minification: bool) -> Vec<u8> {
        if !minification {
            let (Self::CSS(data) | Self::JS(data)) = self;
            return data.as_bytes().to_owned();
        }
        let mut out = Vec::new();
        self.write_into(&mut out).unwrap();
        out
    }

    pub fn write_into<W: std::io::Write>(self, out: &mut W) -> std::io::Result<()> {
        match self {
            Self::CSS(data) => match minifier::css::minify(data) {
                Ok(data) => return data.write(out),
                Err(_) => out.write(data.as_bytes())?,
            },
            Self::JS(data) => return minifier::js::minify(data).write(out),
        };
        Ok(())
    }
}

/// The `Theme` struct should be used instead of the static variables because
/// the `new()` method will look if the user has a theme directory in their
/// source folder and use the users theme instead of the default.
///
/// You should only ever use the static variables directly if you want to
/// override the user's theme with the defaults.
#[derive(Debug, PartialEq)]
pub struct Theme {
    pub index: Vec<u8>,
    pub head: Vec<u8>,
    pub redirect: Vec<u8>,
    pub header: Vec<u8>,
    pub toc_js: Vec<u8>,
    pub toc_html: Vec<u8>,
    pub chrome_css: Vec<u8>,
    pub general_css: Vec<u8>,
    pub print_css: Vec<u8>,
    pub variables_css: Vec<u8>,
    pub fonts_css: Option<Vec<u8>>,
    pub font_files: Vec<PathBuf>,
    pub favicon_png: Option<Vec<u8>>,
    pub favicon_svg: Option<Vec<u8>>,
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
    pub fn new<P: AsRef<Path>>(theme_dir: P, minification: bool) -> Self {
        let theme_dir = theme_dir.as_ref();
        let mut theme = Self::new_with_set_fields(minification);

        // If the theme directory doesn't exist there's no point continuing...
        if !theme_dir.exists() || !theme_dir.is_dir() {
            return theme;
        }

        // Check for individual files, if they exist copy them across
        {
            let files = vec![
                (theme_dir.join("index.hbs"), &mut theme.index),
                (theme_dir.join("head.hbs"), &mut theme.head),
                (theme_dir.join("redirect.hbs"), &mut theme.redirect),
                (theme_dir.join("header.hbs"), &mut theme.header),
                (theme_dir.join("toc.js.hbs"), &mut theme.toc_js),
                (theme_dir.join("toc.html.hbs"), &mut theme.toc_html),
                (theme_dir.join("book.js"), &mut theme.js),
                (theme_dir.join("css/chrome.css"), &mut theme.chrome_css),
                (theme_dir.join("css/general.css"), &mut theme.general_css),
                (theme_dir.join("css/print.css"), &mut theme.print_css),
                (
                    theme_dir.join("css/variables.css"),
                    &mut theme.variables_css,
                ),
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

            let load_with_warn = |filename: &Path, dest: &mut Vec<u8>| {
                if !filename.exists() {
                    // Don't warn if the file doesn't exist.
                    return false;
                }
                if let Err(e) = load_file_contents(filename, dest) {
                    warn!("Couldn't load custom file, {}: {}", filename.display(), e);
                    false
                } else {
                    true
                }
            };

            for (filename, dest) in files {
                load_with_warn(&filename, dest);
            }

            let fonts_dir = theme_dir.join("fonts");
            if fonts_dir.exists() {
                let mut fonts_css = Vec::new();
                if load_with_warn(&fonts_dir.join("fonts.css"), &mut fonts_css) {
                    theme.fonts_css.replace(fonts_css);
                }
                if let Ok(entries) = fonts_dir.read_dir() {
                    theme.font_files = entries
                        .filter_map(|entry| {
                            let entry = entry.ok()?;
                            if entry.file_name() == "fonts.css" {
                                None
                            } else if entry.file_type().ok()?.is_dir() {
                                log::info!("skipping font directory {:?}", entry.path());
                                None
                            } else {
                                Some(entry.path())
                            }
                        })
                        .collect();
                }
            }

            // If the user overrides one favicon, but not the other, do not
            // copy the default for the other.
            let favicon_png = &mut theme.favicon_png.as_mut().unwrap();
            let png = load_with_warn(&theme_dir.join("favicon.png"), favicon_png);
            let favicon_svg = &mut theme.favicon_svg.as_mut().unwrap();
            let svg = load_with_warn(&theme_dir.join("favicon.svg"), favicon_svg);
            match (png, svg) {
                (true, true) | (false, false) => {}
                (true, false) => {
                    theme.favicon_svg = None;
                }
                (false, true) => {
                    theme.favicon_png = None;
                }
            }
        }

        theme
    }

    fn new_with_set_fields(minification: bool) -> Self {
        Theme {
            index: INDEX.to_owned(),
            head: HEAD.to_owned(),
            redirect: REDIRECT.to_owned(),
            header: HEADER.to_owned(),
            toc_js: TOC_JS.to_owned(),
            toc_html: TOC_HTML.to_owned(),
            chrome_css: CHROME_CSS.minified(minification),
            general_css: GENERAL_CSS.minified(minification),
            print_css: PRINT_CSS.minified(minification),
            variables_css: VARIABLES_CSS.minified(minification),
            fonts_css: None,
            font_files: Vec::new(),
            favicon_png: Some(FAVICON_PNG.to_owned()),
            favicon_svg: Some(FAVICON_SVG.to_owned()),
            js: JS.minified(minification),
            highlight_css: HIGHLIGHT_CSS.minified(minification),
            tomorrow_night_css: TOMORROW_NIGHT_CSS.minified(minification),
            ayu_highlight_css: AYU_HIGHLIGHT_CSS.minified(minification),
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
    use tempfile::Builder as TempFileBuilder;

    #[test]
    fn theme_uses_defaults_with_nonexistent_src_dir() {
        let non_existent = PathBuf::from("/non/existent/directory/");
        assert!(!non_existent.exists());

        let minification = false;
        let should_be = Theme::new_with_set_fields(minification);
        let got = Theme::new(&non_existent, minification);

        assert_eq!(got, should_be);
    }

    #[test]
    fn theme_dir_overrides_defaults() {
        let files = [
            "index.hbs",
            "head.hbs",
            "redirect.hbs",
            "header.hbs",
            "toc.js.hbs",
            "toc.html.hbs",
            "favicon.png",
            "favicon.svg",
            "css/chrome.css",
            "css/general.css",
            "css/print.css",
            "css/variables.css",
            "fonts/fonts.css",
            "book.js",
            "highlight.js",
            "tomorrow-night.css",
            "highlight.css",
            "ayu-highlight.css",
            "clipboard.min.js",
        ];

        let temp = TempFileBuilder::new().prefix("mdbook-").tempdir().unwrap();
        fs::create_dir(temp.path().join("css")).unwrap();
        fs::create_dir(temp.path().join("fonts")).unwrap();

        // "touch" all of the special files so we have empty copies
        for file in &files {
            File::create(&temp.path().join(file)).unwrap();
        }

        let got = Theme::new(temp.path(), false);

        let empty = Theme {
            index: Vec::new(),
            head: Vec::new(),
            redirect: Vec::new(),
            header: Vec::new(),
            toc_js: Vec::new(),
            toc_html: Vec::new(),
            chrome_css: Vec::new(),
            general_css: Vec::new(),
            print_css: Vec::new(),
            variables_css: Vec::new(),
            fonts_css: Some(Vec::new()),
            font_files: Vec::new(),
            favicon_png: Some(Vec::new()),
            favicon_svg: Some(Vec::new()),
            js: Vec::new(),
            highlight_css: Vec::new(),
            tomorrow_night_css: Vec::new(),
            ayu_highlight_css: Vec::new(),
            highlight_js: Vec::new(),
            clipboard_js: Vec::new(),
        };

        assert_eq!(got, empty);
    }

    #[test]
    fn favicon_override() {
        let temp = TempFileBuilder::new().prefix("mdbook-").tempdir().unwrap();
        fs::write(temp.path().join("favicon.png"), "1234").unwrap();
        let got = Theme::new(temp.path(), false);
        assert_eq!(got.favicon_png.as_ref().unwrap(), b"1234");
        assert_eq!(got.favicon_svg, None);

        let temp = TempFileBuilder::new().prefix("mdbook-").tempdir().unwrap();
        fs::write(temp.path().join("favicon.svg"), "4567").unwrap();
        let got = Theme::new(temp.path(), false);
        assert_eq!(got.favicon_png, None);
        assert_eq!(got.favicon_svg.as_ref().unwrap(), b"4567");
    }
}
