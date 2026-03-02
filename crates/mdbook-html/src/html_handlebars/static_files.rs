//! Support for writing static files.

use super::helpers::resources::ResourceHelper;
use crate::theme::{self, Theme, playground_editor};
use anyhow::{Context, Result};
use mdbook_core::config::HtmlConfig;
use mdbook_core::static_regex;
use mdbook_core::utils::fs;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Returns the directory component of a forward-slash path (no trailing slash).
/// Returns `""` for paths with no directory component.
fn url_parent_dir(path: &str) -> &str {
    match path.rfind('/') {
        Some(i) => &path[..i],
        None => "",
    }
}

/// Normalizes a slash-separated path by resolving `.` and `..` components.
fn normalize_url_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            c => parts.push(c),
        }
    }
    parts.join("/")
}

/// Expresses `target` as a path relative to `from_dir`.
/// Both are root-relative slash-separated paths.
fn make_url_relative(from_dir: &str, target: &str) -> String {
    let from: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let to: Vec<&str> = target.split('/').filter(|s| !s.is_empty()).collect();
    let common = from
        .iter()
        .zip(to.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let ups = from.len() - common;
    let mut parts: Vec<&str> = Vec::new();
    for _ in 0..ups {
        parts.push("..");
    }
    parts.extend_from_slice(&to[common..]);
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

/// Returns `true` if a CSS `url()` value is absolute and should not be rewritten.
fn is_css_url_absolute(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("data:")
        || url.starts_with("//")
        || url.starts_with('/')
        || url.starts_with('#')
}

/// Map static files to their final names and contents.
///
/// It performs [fingerprinting], if you call the `hash_files` method.
/// If hash-files is turned off, then the files will not be renamed.
/// It also writes files to their final destination, when `write_files` is called,
/// and interprets the `{{ resource }}` directives to allow assets to name each other.
///
/// [fingerprinting]: https://guides.rubyonrails.org/asset_pipeline.html#fingerprinting-versioning-with-digest-based-urls
pub(super) struct StaticFiles {
    static_files: Vec<StaticFile>,
    hash_map: HashMap<String, String>,
}

enum StaticFile {
    Builtin {
        data: Vec<u8>,
        filename: String,
    },
    Additional {
        input_location: PathBuf,
        filename: String,
    },
}

impl StaticFiles {
    pub(super) fn new(theme: &Theme, html_config: &HtmlConfig, root: &Path) -> Result<StaticFiles> {
        let static_files = Vec::new();
        let mut this = StaticFiles {
            hash_map: HashMap::new(),
            static_files,
        };

        this.add_builtin("book.js", &theme.js);
        this.add_builtin("css/general.css", &theme.general_css);
        this.add_builtin("css/chrome.css", &theme.chrome_css);
        if html_config.print.enable {
            this.add_builtin("css/print.css", &theme.print_css);
        }
        this.add_builtin("css/variables.css", &theme.variables_css);
        if let Some(contents) = &theme.favicon_png {
            this.add_builtin("favicon.png", contents);
        }
        if let Some(contents) = &theme.favicon_svg {
            this.add_builtin("favicon.svg", contents);
        }
        this.add_builtin("highlight.css", &theme.highlight_css);
        this.add_builtin("tomorrow-night.css", &theme.tomorrow_night_css);
        this.add_builtin("ayu-highlight.css", &theme.ayu_highlight_css);
        this.add_builtin("highlight.js", &theme.highlight_js);
        this.add_builtin("clipboard.min.js", &theme.clipboard_js);
        if theme.fonts_css.is_none() {
            this.add_builtin("fonts/fonts.css", theme::fonts::CSS);
            for (file_name, contents) in theme::fonts::LICENSES.iter() {
                this.add_builtin(file_name, contents);
            }
            for (file_name, contents) in theme::fonts::OPEN_SANS.iter() {
                this.add_builtin(file_name, contents);
            }
            this.add_builtin(
                theme::fonts::SOURCE_CODE_PRO.0,
                theme::fonts::SOURCE_CODE_PRO.1,
            );
        } else if let Some(fonts_css) = &theme.fonts_css {
            if !fonts_css.is_empty() {
                this.add_builtin("fonts/fonts.css", fonts_css);
            }
        }

        let playground_config = &html_config.playground;

        // Ace is a very large dependency, so only load it when requested
        if playground_config.editable && playground_config.copy_js {
            // Load the editor
            this.add_builtin("editor.js", playground_editor::JS);
            this.add_builtin("ace.js", playground_editor::ACE_JS);
            this.add_builtin("mode-rust.js", playground_editor::MODE_RUST_JS);
            this.add_builtin("theme-dawn.js", playground_editor::THEME_DAWN_JS);
            this.add_builtin(
                "theme-tomorrow_night.js",
                playground_editor::THEME_TOMORROW_NIGHT_JS,
            );
        }

        let custom_files = html_config
            .additional_css
            .iter()
            .chain(html_config.additional_js.iter());

        for custom_file in custom_files {
            let input_location = root.join(custom_file);

            this.static_files.push(StaticFile::Additional {
                input_location,
                filename: custom_file
                    .to_str()
                    .with_context(|| "resource file names must be valid utf8")?
                    .to_owned(),
            });
        }

        for input_location in theme.font_files.iter().cloned() {
            let filename = Path::new("fonts")
                .join(input_location.file_name().unwrap())
                .to_str()
                .with_context(|| "resource file names must be valid utf8")?
                .to_owned();
            this.static_files.push(StaticFile::Additional {
                input_location,
                filename,
            });
        }

        Ok(this)
    }

    pub(super) fn add_builtin(&mut self, filename: &str, data: &[u8]) {
        self.static_files.push(StaticFile::Builtin {
            filename: filename.to_owned(),
            data: data.to_owned(),
        });
    }

    /// Updates this [`StaticFiles`] to hash the contents for determining the
    /// filename for each resource.
    pub(super) fn hash_files(&mut self) -> Result<()> {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        for static_file in &mut self.static_files {
            match static_file {
                &mut StaticFile::Builtin {
                    ref mut filename,
                    ref data,
                } => {
                    let mut parts = filename.splitn(2, '.');
                    let parts = parts.next().and_then(|p| Some((p, parts.next()?)));
                    if let Some((name, suffix)) = parts {
                        if name != "" && suffix != "" && suffix != "txt" {
                            let hex = hex::encode(&Sha256::digest(data)[..4]);
                            let new_filename = format!("{}-{}.{}", name, hex, suffix);
                            self.hash_map.insert(filename.clone(), new_filename.clone());
                            *filename = new_filename;
                        }
                    }
                }
                &mut StaticFile::Additional {
                    ref mut filename,
                    ref input_location,
                } => {
                    let mut parts = filename.splitn(2, '.');
                    let parts = parts.next().and_then(|p| Some((p, parts.next()?)));
                    if let Some((name, suffix)) = parts {
                        if name != "" && suffix != "" {
                            let mut digest = Sha256::new();
                            let mut input_file =
                                std::fs::File::open(input_location).with_context(|| {
                                    format!("failed to open `{filename}` for hashing")
                                })?;
                            let mut buf = vec![0; 1024];
                            loop {
                                let amt = input_file
                                    .read(&mut buf)
                                    .with_context(|| "read static file for hashing")?;
                                if amt == 0 {
                                    break;
                                };
                                digest.update(&buf[..amt]);
                            }
                            let hex = hex::encode(&digest.finalize()[..4]);
                            let new_filename = format!("{}-{}.{}", name, hex, suffix);
                            self.hash_map.insert(filename.clone(), new_filename.clone());
                            *filename = new_filename;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn write_files(self, destination: &Path) -> Result<ResourceHelper> {
        use regex::bytes::Captures;
        // The `{{ resource "name" }}` directive in static resources look like
        // handlebars syntax, even if they technically aren't.
        static_regex!(RESOURCE, bytes, r#"\{\{ resource "([^"]+)" \}\}"#);
        // CSS url() with double-quoted, single-quoted, or unquoted paths.
        // Capture groups: 1=double-quoted path, 2=single-quoted path, 3=unquoted path.
        static_regex!(
            CSS_URL,
            bytes,
            r#"url\(\s*(?:"([^"]*?)"|'([^']*?)'|([^'"\s()]+))\s*\)"#
        );
        fn replace_all<'a>(
            hash_map: &HashMap<String, String>,
            data: &'a [u8],
            filename: &str,
        ) -> Cow<'a, [u8]> {
            let data = RESOURCE.replace_all(data, move |captures: &Captures<'_>| {
                let name = captures
                    .get(1)
                    .expect("capture 1 in resource regex")
                    .as_bytes();
                let name = std::str::from_utf8(name).expect("resource name with invalid utf8");
                let resource_filename = hash_map.get(name).map(|s| &s[..]).unwrap_or(name);
                let path_to_root = fs::path_to_root(filename);
                format!("{}{}", path_to_root, resource_filename)
                    .as_bytes()
                    .to_owned()
            });
            // Convert to owned to break the borrow chain before the second pass.
            let data = data.into_owned();
            let css_dir = url_parent_dir(filename);
            let data = CSS_URL.replace_all(&data, move |captures: &Captures<'_>| {
                let (url_bytes, quote) = if let Some(m) = captures.get(1) {
                    (m.as_bytes(), "\"")
                } else if let Some(m) = captures.get(2) {
                    (m.as_bytes(), "'")
                } else {
                    (
                        captures
                            .get(3)
                            .expect("capture group 3 in CSS url regex")
                            .as_bytes(),
                        "",
                    )
                };
                let url_str = match std::str::from_utf8(url_bytes) {
                    Ok(s) => s,
                    Err(_) => return captures[0].to_owned(),
                };
                if is_css_url_absolute(url_str) {
                    return captures[0].to_owned();
                }
                // Resolve the URL relative to the CSS file's directory to get
                // the root-relative path for hash_map lookup.
                let resolved = if css_dir.is_empty() {
                    normalize_url_path(url_str)
                } else {
                    normalize_url_path(&format!("{}/{}", css_dir, url_str))
                };
                if let Some(hashed) = hash_map.get(&resolved) {
                    // Express the hashed path relative to the CSS file's directory.
                    let relative = if css_dir.is_empty() {
                        hashed.clone()
                    } else {
                        make_url_relative(css_dir, hashed)
                    };
                    format!("url({quote}{relative}{quote})")
                        .as_bytes()
                        .to_owned()
                } else {
                    captures[0].to_owned()
                }
            });
            Cow::Owned(data.into_owned())
        }
        for static_file in &self.static_files {
            match static_file {
                StaticFile::Builtin { filename, data } => {
                    debug!("Writing builtin -> {}", filename);
                    let data = if filename.ends_with(".css") || filename.ends_with(".js") {
                        replace_all(&self.hash_map, data, filename)
                    } else {
                        Cow::Borrowed(&data[..])
                    };
                    let path = destination.join(filename);
                    fs::write(path, &data)?;
                }
                StaticFile::Additional {
                    input_location,
                    filename,
                } => {
                    let output_location = destination.join(filename);
                    debug!(
                        "Copying {} -> {}",
                        input_location.display(),
                        output_location.display()
                    );
                    if let Some(parent) = output_location.parent() {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("Unable to create {}", parent.display()))?;
                    }
                    if filename.ends_with(".css") || filename.ends_with(".js") {
                        let data = fs::read_to_string(input_location)?;
                        let data = replace_all(&self.hash_map, data.as_bytes(), filename);
                        let path = destination.join(filename);
                        fs::write(path, &data)?;
                    } else {
                        std::fs::copy(input_location, &output_location).with_context(|| {
                            format!(
                                "Unable to copy {} to {}",
                                input_location.display(),
                                output_location.display()
                            )
                        })?;
                    }
                }
            }
        }
        let hash_map = self.hash_map;
        Ok(ResourceHelper { hash_map })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use mdbook_core::config::HtmlConfig;
    use mdbook_core::utils::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_directive() {
        let theme = Theme {
            index: Vec::new(),
            head: Vec::new(),
            redirect: Vec::new(),
            header: Vec::new(),
            chrome_css: Vec::new(),
            general_css: Vec::new(),
            print_css: Vec::new(),
            variables_css: Vec::new(),
            favicon_png: Some(Vec::new()),
            favicon_svg: Some(Vec::new()),
            js: Vec::new(),
            highlight_css: Vec::new(),
            tomorrow_night_css: Vec::new(),
            ayu_highlight_css: Vec::new(),
            highlight_js: Vec::new(),
            clipboard_js: Vec::new(),
            toc_js: Vec::new(),
            toc_html: Vec::new(),
            fonts_css: None,
            font_files: Vec::new(),
        };
        let temp_dir = TempDir::with_prefix("mdbook-").unwrap();
        let reference_js = Path::new("static-files-test-case-reference.js");
        let mut html_config = HtmlConfig::default();
        html_config.additional_js.push(reference_js.to_owned());
        fs::write(
            temp_dir.path().join(reference_js),
            br#"{{ resource "book.js" }}"#,
        )
        .unwrap();
        let mut static_files = StaticFiles::new(&theme, &html_config, temp_dir.path()).unwrap();
        static_files.hash_files().unwrap();
        static_files.write_files(temp_dir.path()).unwrap();
        // custom JS winds up referencing book.js
        let reference_js_content = fs::read_to_string(
            temp_dir
                .path()
                .join("static-files-test-case-reference-635c9cdc.js"),
        )
        .unwrap();
        assert_eq!("book-e3b0c442.js", reference_js_content);
        // book.js winds up empty
        let book_js_content = fs::read_to_string(temp_dir.path().join("book-e3b0c442.js")).unwrap();
        assert_eq!("", book_js_content);
    }

    // ── helper ──────────────────────────────────────────────────────────────
    /// Sets up a temp dir with a theme font at `theme/fonts/test-font.woff2`
    /// and returns (temp_dir, theme).
    fn setup_theme_with_font() -> (TempDir, Theme) {
        let temp_dir = TempDir::with_prefix("mdbook-").unwrap();
        let theme_dir = temp_dir.path().join("theme");
        fs::create_dir_all(theme_dir.join("fonts")).unwrap();
        fs::write(theme_dir.join("fonts/test-font.woff2"), b"font-data").unwrap();
        let theme = Theme::new(&theme_dir);
        (temp_dir, theme)
    }

    /// Reads the content of the single CSS file whose name starts with `prefix`
    /// inside `dir`.  Panics if none or more than one is found.
    fn read_hashed_css(dir: &Path, prefix: &str) -> String {
        let matches: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|name| name.starts_with(prefix) && name.ends_with(".css"))
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "expected exactly one {prefix}*.css, got {matches:?}"
        );
        fs::read_to_string(dir.join(&matches[0])).unwrap()
    }

    // ── unit tests for URL helper functions ─────────────────────────────────

    #[test]
    fn test_url_parent_dir() {
        assert_eq!(url_parent_dir("css/custom.css"), "css");
        assert_eq!(url_parent_dir("a/b/c.css"), "a/b");
        assert_eq!(url_parent_dir("custom.css"), "");
        assert_eq!(url_parent_dir(""), "");
    }

    #[test]
    fn test_normalize_url_path() {
        assert_eq!(normalize_url_path("fonts/test.woff2"), "fonts/test.woff2");
        assert_eq!(
            normalize_url_path("css/../fonts/test.woff2"),
            "fonts/test.woff2"
        );
        assert_eq!(normalize_url_path("a/b/../../c"), "c");
        assert_eq!(normalize_url_path("./fonts/test.woff2"), "fonts/test.woff2");
        assert_eq!(normalize_url_path("fonts//test.woff2"), "fonts/test.woff2");
    }

    #[test]
    fn test_make_url_relative() {
        // CSS in css/, font in fonts/ → go up one, then into fonts/
        assert_eq!(
            make_url_relative("css", "fonts/test-abc.woff2"),
            "../fonts/test-abc.woff2"
        );
        // CSS in fonts/, font also in fonts/ → same directory
        assert_eq!(
            make_url_relative("fonts", "fonts/test-abc.woff2"),
            "test-abc.woff2"
        );
        // CSS in a/b/, font in a/c/ → sibling directory
        assert_eq!(make_url_relative("a/b", "a/c/d.woff2"), "../c/d.woff2");
        // Same directory, different file
        assert_eq!(make_url_relative("css", "css/other.css"), "other.css");
    }

    // ── integration tests for CSS url() rewriting ────────────────────────────

    /// Regression test for https://github.com/rust-lang/mdBook/issues/2958:
    /// a root-level CSS with a double-quoted url() reference to a custom font.
    #[test]
    fn test_css_url_double_quoted() {
        let (temp_dir, theme) = setup_theme_with_font();

        let custom_css = Path::new("custom.css");
        fs::write(
            temp_dir.path().join(custom_css),
            br#"@font-face { src: url("fonts/test-font.woff2") format("woff2"); }"#,
        )
        .unwrap();

        let mut html_config = HtmlConfig::default();
        html_config.additional_css.push(custom_css.to_owned());

        let mut sf = StaticFiles::new(&theme, &html_config, temp_dir.path()).unwrap();
        sf.hash_files().unwrap();
        sf.write_files(temp_dir.path()).unwrap();

        let content = read_hashed_css(temp_dir.path(), "custom-");
        assert!(
            content.contains("url(\"fonts/test-font-"),
            "url not rewritten: {content}"
        );
        assert!(!content.contains("url(\"fonts/test-font.woff2\")"));
    }

    /// Single-quoted `url('…')` references are rewritten.
    #[test]
    fn test_css_url_single_quoted() {
        let (temp_dir, theme) = setup_theme_with_font();

        let custom_css = Path::new("custom.css");
        fs::write(
            temp_dir.path().join(custom_css),
            br#"@font-face { src: url('fonts/test-font.woff2') format('woff2'); }"#,
        )
        .unwrap();

        let mut html_config = HtmlConfig::default();
        html_config.additional_css.push(custom_css.to_owned());

        let mut sf = StaticFiles::new(&theme, &html_config, temp_dir.path()).unwrap();
        sf.hash_files().unwrap();
        sf.write_files(temp_dir.path()).unwrap();

        let content = read_hashed_css(temp_dir.path(), "custom-");
        assert!(
            content.contains("url('fonts/test-font-"),
            "url not rewritten: {content}"
        );
        assert!(!content.contains("url('fonts/test-font.woff2')"));
    }

    /// Unquoted `url(…)` references are rewritten.
    #[test]
    fn test_css_url_unquoted() {
        let (temp_dir, theme) = setup_theme_with_font();

        let custom_css = Path::new("custom.css");
        fs::write(
            temp_dir.path().join(custom_css),
            b"@font-face { src: url(fonts/test-font.woff2); }",
        )
        .unwrap();

        let mut html_config = HtmlConfig::default();
        html_config.additional_css.push(custom_css.to_owned());

        let mut sf = StaticFiles::new(&theme, &html_config, temp_dir.path()).unwrap();
        sf.hash_files().unwrap();
        sf.write_files(temp_dir.path()).unwrap();

        let content = read_hashed_css(temp_dir.path(), "custom-");
        assert!(
            content.contains("url(fonts/test-font-"),
            "url not rewritten: {content}"
        );
        assert!(!content.contains("url(fonts/test-font.woff2)"));
    }

    /// A CSS file in a subdirectory uses a path relative to itself (`../fonts/…`).
    /// The rewritten URL must also be relative to the CSS file's location.
    #[test]
    fn test_css_url_subdirectory() {
        let (temp_dir, theme) = setup_theme_with_font();

        let custom_css = Path::new("css/custom.css");
        fs::create_dir_all(temp_dir.path().join("css")).unwrap();
        // Correct relative path from css/ to fonts/ is ../fonts/
        fs::write(
            temp_dir.path().join(custom_css),
            br#"@font-face { src: url("../fonts/test-font.woff2") format("woff2"); }"#,
        )
        .unwrap();

        let mut html_config = HtmlConfig::default();
        html_config.additional_css.push(custom_css.to_owned());

        let mut sf = StaticFiles::new(&theme, &html_config, temp_dir.path()).unwrap();
        sf.hash_files().unwrap();
        sf.write_files(temp_dir.path()).unwrap();

        let content = read_hashed_css(&temp_dir.path().join("css"), "custom-");
        // The rewritten URL must still be relative to css/, so ../fonts/test-font-<hash>.woff2
        assert!(
            content.contains("url(\"../fonts/test-font-"),
            "url not rewritten: {content}"
        );
        assert!(!content.contains("url(\"../fonts/test-font.woff2\")"));
    }

    /// Absolute URLs (`https://`, `data:`, `//`, `/`) are left untouched.
    #[test]
    fn test_css_url_absolute_unchanged() {
        let (temp_dir, theme) = setup_theme_with_font();

        let css_content = concat!(
            r#"@import url("https://fonts.googleapis.com/css2?family=Test");"#,
            "\n",
            r#"div { background: url("//cdn.example.com/img.png"); }"#,
            "\n",
            r#"div { background: url("/absolute/path.png"); }"#,
            "\n",
            r#"div { background: url("data:image/png;base64,abc"); }"#,
        );

        let custom_css = Path::new("custom.css");
        fs::write(temp_dir.path().join(custom_css), css_content.as_bytes()).unwrap();

        let mut html_config = HtmlConfig::default();
        html_config.additional_css.push(custom_css.to_owned());

        let mut sf = StaticFiles::new(&theme, &html_config, temp_dir.path()).unwrap();
        sf.hash_files().unwrap();
        sf.write_files(temp_dir.path()).unwrap();

        let content = read_hashed_css(temp_dir.path(), "custom-");
        assert!(content.contains("url(\"https://fonts.googleapis.com/css2?family=Test\")"));
        assert!(content.contains("url(\"//cdn.example.com/img.png\")"));
        assert!(content.contains("url(\"/absolute/path.png\")"));
        assert!(content.contains("url(\"data:image/png;base64,abc\")"));
    }

    /// A `url()` path that is not a hashed asset (not in the hash map) is left untouched.
    #[test]
    fn test_css_url_unknown_path_unchanged() {
        let (temp_dir, theme) = setup_theme_with_font();

        let custom_css = Path::new("custom.css");
        fs::write(
            temp_dir.path().join(custom_css),
            br#"div { background: url("images/bg.png"); }"#,
        )
        .unwrap();

        let mut html_config = HtmlConfig::default();
        html_config.additional_css.push(custom_css.to_owned());

        let mut sf = StaticFiles::new(&theme, &html_config, temp_dir.path()).unwrap();
        sf.hash_files().unwrap();
        sf.write_files(temp_dir.path()).unwrap();

        let content = read_hashed_css(temp_dir.path(), "custom-");
        assert!(
            content.contains("url(\"images/bg.png\")"),
            "should be unchanged: {content}"
        );
    }
}
