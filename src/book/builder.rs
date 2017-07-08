use std::path::{Path, PathBuf};

use config::{self, BookConfig};
use renderer::{Renderer, HtmlHandlebars};
use loader;
use errors::*;
use super::MDBook;


#[derive(Default)]
pub struct Builder {
    root: PathBuf,
    create_missing: bool,
    config: Option<BookConfig>,
    renderer: Option<Box<Renderer>>,
    livereload: Option<String>,
}

impl Builder {
    /// Create a new builder which loads the book from an existing directory.
    pub fn new<P: AsRef<Path>>(root: P) -> Builder {
        let root = root.as_ref();

        Builder {
            root: root.to_path_buf(),
            ..Default::default()
        }
    }

    /// Set the config to use.
    pub fn with_config(mut self, config: BookConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<MDBook> {
        // if no custom config provided, try to read it from disk
        let cfg = match self.config {
            Some(c) => c,
            None => config::read_config(&self.root)?,
        };

        let book = loader::load_book(cfg.get_source())?;
        let renderer: Box<Renderer> = self.renderer.unwrap_or_else(
            || Box::new(HtmlHandlebars::new()),
        );

        Ok(MDBook {
            config: cfg,
            book: book,
            renderer: renderer,
            livereload: self.livereload,
            create_missing: self.create_missing,
        })
    }
}