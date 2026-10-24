//! Support for theme files.

use anyhow::Result;
use mdbook_core::config::HtmlConfig;
use mdbook_core::utils::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub(crate) mod fonts;
pub(crate) mod playground_editor;
#[cfg(feature = "search")]
pub(crate) mod searcher;

static INDEX: &[u8] = include_bytes!("../../front-end/templates/index.hbs");
static HEAD: &[u8] = include_bytes!("../../front-end/templates/head.hbs");
static REDIRECT: &[u8] = include_bytes!("../../front-end/templates/redirect.hbs");
static HEADER: &[u8] = include_bytes!("../../front-end/templates/header.hbs");
static TOC_JS: &[u8] = include_bytes!("../../front-end/templates/toc.js.hbs");
static TOC_HTML: &[u8] = include_bytes!("../../front-end/templates/toc.html.hbs");
static CHROME_CSS: &[u8] = include_bytes!("../../front-end/css/chrome.css");
static GENERAL_CSS: &[u8] = include_bytes!("../../front-end/css/general.css");
static PRINT_CSS: &[u8] = include_bytes!("../../front-end/css/print.css");
static VARIABLES_CSS: &[u8] = include_bytes!("../../front-end/css/variables.css");
static FAVICON_PNG: &[u8] = include_bytes!("../../front-end/images/favicon.png");
static FAVICON_SVG: &[u8] = include_bytes!("../../front-end/images/favicon.svg");
static JS: &[u8] = include_bytes!("../../front-end/js/book.js");
static HIGHLIGHT_JS: &[u8] = include_bytes!("../../front-end/js/highlight.js");
static TOMORROW_NIGHT_CSS: &[u8] = include_bytes!("../../front-end/css/tomorrow-night.css");
static HIGHLIGHT_CSS: &[u8] = include_bytes!("../../front-end/css/highlight.css");
static AYU_HIGHLIGHT_CSS: &[u8] = include_bytes!("../../front-end/css/ayu-highlight.css");
static CLIPBOARD_JS: &[u8] = include_bytes!("../../front-end/js/clipboard.min.js");

/// The `Theme` struct should be used instead of the static variables because
/// the `new()` method will look if the user has a theme directory in their
/// source folder and use the users theme instead of the default.
///
/// You should only ever use the static variables directly if you want to
/// override the user's theme with the defaults.
#[derive(Debug, PartialEq)]
pub struct Theme {
    pub(crate) index: Vec<u8>,
    pub(crate) head: Vec<u8>,
    pub(crate) redirect: Vec<u8>,
    pub(crate) header: Vec<u8>,
    pub(crate) toc_js: Vec<u8>,
    pub(crate) toc_html: Vec<u8>,
    pub(crate) chrome_css: Vec<u8>,
    pub(crate) general_css: Vec<u8>,
    pub(crate) print_css: Vec<u8>,
    pub(crate) variables_css: Vec<u8>,
    pub(crate) fonts_css: Option<Vec<u8>>,
    pub(crate) font_files: Vec<PathBuf>,
    pub(crate) favicon_png: Option<Vec<u8>>,
    pub(crate) favicon_svg: Option<Vec<u8>>,
    pub(crate) js: Vec<u8>,
    pub(crate) highlight_css: Vec<u8>,
    pub(crate) tomorrow_night_css: Vec<u8>,
    pub(crate) ayu_highlight_css: Vec<u8>,
    pub(crate) highlight_js: Vec<u8>,
    pub(crate) clipboard_js: Vec<u8>,
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
                                info!("skipping font directory {:?}", entry.path());
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

    /// Copies the default theme files to the theme directory.
    pub fn copy_theme(html_config: &HtmlConfig, root: &Path) -> Result<()> {
        let themedir = html_config.theme_dir(root);

        fs::write(themedir.join("book.js"), JS)?;
        fs::write(themedir.join("favicon.png"), FAVICON_PNG)?;
        fs::write(themedir.join("favicon.svg"), FAVICON_SVG)?;
        fs::write(themedir.join("highlight.css"), HIGHLIGHT_CSS)?;
        fs::write(themedir.join("highlight.js"), HIGHLIGHT_JS)?;
        fs::write(themedir.join("index.hbs"), INDEX)?;

        let cssdir = themedir.join("css");

        fs::write(cssdir.join("general.css"), GENERAL_CSS)?;
        fs::write(cssdir.join("chrome.css"), CHROME_CSS)?;
        fs::write(cssdir.join("variables.css"), VARIABLES_CSS)?;
        if html_config.print.enable {
            fs::write(cssdir.join("print.css"), PRINT_CSS)?;
        }

        fs::write(themedir.join("fonts").join("fonts.css"), fonts::CSS)?;
        for (file_name, contents) in fonts::LICENSES {
            fs::write(themedir.join(file_name), contents)?;
        }
        for (file_name, contents) in fonts::OPEN_SANS.iter() {
            fs::write(themedir.join(file_name), contents)?;
        }
        fs::write(
            themedir.join(fonts::SOURCE_CODE_PRO.0),
            fonts::SOURCE_CODE_PRO.1,
        )?;
        Ok(())
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme {
            index: INDEX.to_owned(),
            head: HEAD.to_owned(),
            redirect: REDIRECT.to_owned(),
            header: HEADER.to_owned(),
            toc_js: TOC_JS.to_owned(),
            toc_html: TOC_HTML.to_owned(),
            chrome_css: CHROME_CSS.to_owned(),
            general_css: GENERAL_CSS.to_owned(),
            print_css: PRINT_CSS.to_owned(),
            variables_css: VARIABLES_CSS.to_owned(),
            fonts_css: None,
            font_files: Vec::new(),
            favicon_png: Some(FAVICON_PNG.to_owned()),
            favicon_svg: Some(FAVICON_SVG.to_owned()),
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
    let mut buffer = std::fs::read(filename)?;

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

        let should_be = Theme::default();
        let got = Theme::new(&non_existent);

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
            fs::File::create(&temp.path().join(file)).unwrap();
        }

        let got = Theme::new(temp.path());

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
        let got = Theme::new(temp.path());
        assert_eq!(got.favicon_png.as_ref().unwrap(), b"1234");
        assert_eq!(got.favicon_svg, None);

        let temp = TempFileBuilder::new().prefix("mdbook-").tempdir().unwrap();
        fs::write(temp.path().join("favicon.svg"), "4567").unwrap();
        let got = Theme::new(temp.path());
        assert_eq!(got.favicon_png, None);
        assert_eq!(got.favicon_svg.as_ref().unwrap(), b"4567");
    }
}
