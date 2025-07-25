//! Support for writing static files.

use super::helpers::resources::ResourceHelper;
use crate::theme::{self, Theme, playground_editor};
use anyhow::{Context, Result};
use log::{debug, warn};
use mdbook_core::config::HtmlConfig;
use mdbook_core::utils;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

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
        this.add_builtin("FontAwesome/css/font-awesome.css", theme::FONT_AWESOME);
        this.add_builtin(
            "FontAwesome/fonts/fontawesome-webfont.eot",
            theme::FONT_AWESOME_EOT,
        );
        this.add_builtin(
            "FontAwesome/fonts/fontawesome-webfont.svg",
            theme::FONT_AWESOME_SVG,
        );
        this.add_builtin(
            "FontAwesome/fonts/fontawesome-webfont.ttf",
            theme::FONT_AWESOME_TTF,
        );
        this.add_builtin(
            "FontAwesome/fonts/fontawesome-webfont.woff",
            theme::FONT_AWESOME_WOFF,
        );
        this.add_builtin(
            "FontAwesome/fonts/fontawesome-webfont.woff2",
            theme::FONT_AWESOME_WOFF2,
        );
        this.add_builtin("FontAwesome/fonts/FontAwesome.ttf", theme::FONT_AWESOME_TTF);
        if html_config.copy_fonts && theme.fonts_css.is_none() {
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
        if !html_config.copy_fonts && theme.fonts_css.is_none() {
            warn!(
                "output.html.copy-fonts is deprecated.\n\
                This book appears to have copy-fonts=false in book.toml without a fonts.css file.\n\
                Add an empty `theme/fonts/fonts.css` file to squelch this warning."
            );
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
                        // FontAwesome already does its own cache busting with the ?v=4.7.0 thing,
                        // and I don't want to have to patch its CSS file to use `{{ resource }}`
                        if name != ""
                            && suffix != ""
                            && suffix != "txt"
                            && !name.starts_with("FontAwesome/fonts/")
                        {
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
                            let mut input_file = File::open(input_location)
                                .with_context(|| "open static file for hashing")?;
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
        use mdbook_core::utils::fs::write_file;
        use regex::bytes::{Captures, Regex};
        // The `{{ resource "name" }}` directive in static resources look like
        // handlebars syntax, even if they technically aren't.
        static RESOURCE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"\{\{ resource "([^"]+)" \}\}"#).unwrap());
        fn replace_all<'a>(
            hash_map: &HashMap<String, String>,
            data: &'a [u8],
            filename: &str,
        ) -> Cow<'a, [u8]> {
            RESOURCE.replace_all(data, move |captures: &Captures<'_>| {
                let name = captures
                    .get(1)
                    .expect("capture 1 in resource regex")
                    .as_bytes();
                let name = std::str::from_utf8(name).expect("resource name with invalid utf8");
                let resource_filename = hash_map.get(name).map(|s| &s[..]).unwrap_or(name);
                let path_to_root = utils::fs::path_to_root(filename);
                format!("{}{}", path_to_root, resource_filename)
                    .as_bytes()
                    .to_owned()
            })
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
                    write_file(destination, filename, &data)?;
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
                        let data = fs::read(input_location)?;
                        let data = replace_all(&self.hash_map, &data, filename);
                        write_file(destination, filename, &data)?;
                    } else {
                        fs::copy(input_location, &output_location).with_context(|| {
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
    use mdbook_core::utils::fs::write_file;
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
        write_file(
            temp_dir.path(),
            reference_js,
            br#"{{ resource "book.js" }}"#,
        )
        .unwrap();
        let mut static_files = StaticFiles::new(&theme, &html_config, temp_dir.path()).unwrap();
        static_files.hash_files().unwrap();
        static_files.write_files(temp_dir.path()).unwrap();
        // custom JS winds up referencing book.js
        let reference_js_content = std::fs::read_to_string(
            temp_dir
                .path()
                .join("static-files-test-case-reference-635c9cdc.js"),
        )
        .unwrap();
        assert_eq!("book-e3b0c442.js", reference_js_content);
        // book.js winds up empty
        let book_js_content =
            std::fs::read_to_string(temp_dir.path().join("book-e3b0c442.js")).unwrap();
        assert_eq!("", book_js_content);
    }
}
