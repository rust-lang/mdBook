//! Support for initializing a new book.

use super::MDBook;
use anyhow::{Context, Result};
use mdbook_core::config::Config;
use mdbook_core::utils::fs;
use mdbook_html::theme;
use std::path::PathBuf;
use tracing::{debug, error, info, trace};

/// A helper for setting up a new book and its directory structure.
#[derive(Debug, Clone, PartialEq)]
pub struct BookBuilder {
    root: PathBuf,
    create_gitignore: bool,
    config: Config,
    copy_theme: bool,
}

impl BookBuilder {
    /// Create a new `BookBuilder` which will generate a book in the provided
    /// root directory.
    pub fn new<P: Into<PathBuf>>(root: P) -> BookBuilder {
        BookBuilder {
            root: root.into(),
            create_gitignore: false,
            config: Config::default(),
            copy_theme: false,
        }
    }

    /// Set the [`Config`] to be used.
    pub fn with_config(&mut self, cfg: Config) -> &mut BookBuilder {
        self.config = cfg;
        self
    }

    /// Get the config used by the `BookBuilder`.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Should the theme be copied into the generated book (so users can tweak
    /// it)?
    pub fn copy_theme(&mut self, copy: bool) -> &mut BookBuilder {
        self.copy_theme = copy;
        self
    }

    /// Should we create a `.gitignore` file?
    pub fn create_gitignore(&mut self, create: bool) -> &mut BookBuilder {
        self.create_gitignore = create;
        self
    }

    /// Generate the actual book. This will:
    ///
    /// - Create the directory structure.
    /// - Stub out some dummy chapters and the `SUMMARY.md`.
    /// - Create a `.gitignore` (if applicable)
    /// - Create a themes directory and populate it (if applicable)
    /// - Generate a `book.toml` file,
    /// - Then load the book so we can build it or run tests.
    pub fn build(&self) -> Result<MDBook> {
        info!("Creating a new book with stub content");

        self.create_directory_structure()
            .with_context(|| "Unable to create directory structure")?;

        self.create_stub_files()
            .with_context(|| "Unable to create stub files")?;

        if self.create_gitignore {
            self.build_gitignore()
                .with_context(|| "Unable to create .gitignore")?;
        }

        if self.copy_theme {
            self.copy_across_theme()
                .with_context(|| "Unable to copy across the theme")?;
        }

        self.write_book_toml()?;

        match MDBook::load(&self.root) {
            Ok(book) => Ok(book),
            Err(e) => {
                error!("{}", e);

                panic!(
                    "The BookBuilder should always create a valid book. If you are seeing this it \
                     is a bug and should be reported."
                );
            }
        }
    }

    fn write_book_toml(&self) -> Result<()> {
        debug!("Writing book.toml");
        let book_toml = self.root.join("book.toml");
        let cfg =
            toml::to_string(&self.config).with_context(|| "Unable to serialize the config")?;

        fs::write(&book_toml, cfg)?;
        Ok(())
    }

    fn copy_across_theme(&self) -> Result<()> {
        debug!("Copying theme");

        let html_config = self.config.html_config().unwrap_or_default();
        let themedir = html_config.theme_dir(&self.root);

        fs::write(themedir.join("book.js"), theme::JS)?;
        fs::write(themedir.join("favicon.png"), theme::FAVICON_PNG)?;
        fs::write(themedir.join("favicon.svg"), theme::FAVICON_SVG)?;
        fs::write(themedir.join("highlight.css"), theme::HIGHLIGHT_CSS)?;
        fs::write(themedir.join("highlight.js"), theme::HIGHLIGHT_JS)?;
        fs::write(themedir.join("index.hbs"), theme::INDEX)?;

        let cssdir = themedir.join("css");

        fs::write(cssdir.join("general.css"), theme::GENERAL_CSS)?;
        fs::write(cssdir.join("chrome.css"), theme::CHROME_CSS)?;
        fs::write(cssdir.join("variables.css"), theme::VARIABLES_CSS)?;
        if html_config.print.enable {
            fs::write(cssdir.join("print.css"), theme::PRINT_CSS)?;
        }

        let fonts_dir = themedir.join("fonts");
        fs::write(fonts_dir.join("fonts.css"), theme::fonts::CSS)?;
        for (file_name, contents) in theme::fonts::LICENSES {
            fs::write(themedir.join(file_name), contents)?;
        }
        for (file_name, contents) in theme::fonts::OPEN_SANS.iter() {
            fs::write(themedir.join(file_name), contents)?;
        }
        fs::write(
            themedir.join(theme::fonts::SOURCE_CODE_PRO.0),
            theme::fonts::SOURCE_CODE_PRO.1,
        )?;

        Ok(())
    }

    fn build_gitignore(&self) -> Result<()> {
        fs::write(
            self.root.join(".gitignore"),
            format!("{}", self.config.build.build_dir.display()),
        )?;
        Ok(())
    }

    fn create_stub_files(&self) -> Result<()> {
        debug!("Creating example book contents");
        let src_dir = self.root.join(&self.config.book.src);

        let summary = src_dir.join("SUMMARY.md");
        if !summary.exists() {
            trace!("No summary found creating stub summary and chapter_1.md.");
            fs::write(
                summary,
                "# Summary\n\
                 \n\
                 - [Chapter 1](./chapter_1.md)\n",
            )?;

            fs::write(src_dir.join("chapter_1.md"), "# Chapter 1\n")?;
        } else {
            trace!("Existing summary found, no need to create stub files.");
        }
        Ok(())
    }

    fn create_directory_structure(&self) -> Result<()> {
        debug!("Creating directory tree");
        fs::create_dir_all(&self.root)?;

        let src = self.root.join(&self.config.book.src);
        fs::create_dir_all(src)?;

        let build = self.root.join(&self.config.build.build_dir);
        fs::create_dir_all(build)?;

        Ok(())
    }
}
