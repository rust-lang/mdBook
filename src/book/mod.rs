//! The internal representation of a book and infrastructure for loading it from
//! disk and building it.
//!
//! For examples on using `MDBook`, consult the [top-level documentation][1].
//!
//! [1]: ../index.html

#![deny(missing_docs)]

mod summary;
mod book;
mod init;

pub use self::book::{load_book, Book, BookItem, BookItems, Chapter};
pub use self::summary::{parse_summary, Link, SectionNumber, Summary, SummaryItem};
pub use self::init::BookBuilder;

use std::path::{Path, PathBuf};
use std::io::Write;
use std::process::Command;
use tempdir::TempDir;

use utils;
use renderer::{HtmlHandlebars, Renderer};
use preprocess;
use errors::*;

use config::Config;

/// The object used to manage and build a book.
pub struct MDBook {
    /// The book's root directory.
    pub root: PathBuf,
    /// The configuration used to tweak now a book is built.
    pub config: Config,

    book: Book,
    renderer: Box<Renderer>,

    /// The URL used for live reloading when serving up the book.
    pub livereload: Option<String>,
}

impl MDBook {
    /// Load a book from its root directory on disk.
    pub fn load<P: Into<PathBuf>>(book_root: P) -> Result<MDBook> {
        let book_root = book_root.into();
        let config_location = book_root.join("book.toml");

        // the book.json file is no longer used, so we should emit a warning to
        // let people know to migrate to book.toml
        if book_root.join("book.json").exists() {
            warn!("It appears you are still using book.json for configuration.");
            warn!("This format is no longer used, so you should migrate to the");
            warn!("book.toml format.");
            warn!("Check the user guide for migration information:");
            warn!("\thttps://rust-lang-nursery.github.io/mdBook/format/config.html");
        }

        let config = if config_location.exists() {
            debug!("[*] Loading config from {}", config_location.display());
            Config::from_disk(&config_location)?
        } else {
            Config::default()
        };

        if log_enabled!(::log::LogLevel::Trace) {
            for line in format!("Config: {:#?}", config).lines() {
                trace!("{}", line);
            }
        }

        MDBook::load_with_config(book_root, config)
    }

    /// Load a book from its root directory using a custom config.
    pub fn load_with_config<P: Into<PathBuf>>(book_root: P, config: Config) -> Result<MDBook> {
        let book_root = book_root.into();

        let src_dir = book_root.join(&config.book.src);
        let book = book::load_book(&src_dir, &config.build)?;

        Ok(MDBook {
            root: book_root,
            config: config,
            book: book,
            renderer: Box::new(HtmlHandlebars::new()),
            livereload: None,
        })
    }

    /// Returns a flat depth-first iterator over the elements of the book,
    /// it returns an [BookItem enum](bookitem.html):
    /// `(section: String, bookitem: &BookItem)`
    ///
    /// ```no_run
    /// # extern crate mdbook;
    /// # use mdbook::MDBook;
    /// # use mdbook::book::BookItem;
    /// # #[allow(unused_variables)]
    /// # fn main() {
    /// # let book = MDBook::load("mybook").unwrap();
    /// for item in book.iter() {
    ///     match *item {
    ///         BookItem::Chapter(ref chapter) => {},
    ///         BookItem::Separator => {},
    ///     }
    /// }
    ///
    /// // would print something like this:
    /// // 1. Chapter 1
    /// // 1.1 Sub Chapter
    /// // 1.2 Sub Chapter
    /// // 2. Chapter 2
    /// //
    /// // etc.
    /// # }
    /// ```
    pub fn iter(&self) -> BookItems {
        self.book.iter()
    }

    /// `init()` gives you a `BookBuilder` which you can use to setup a new book
    /// and its accompanying directory structure.
    ///
    /// The `BookBuilder` creates some boilerplate files and directories to get
    /// you started with your book.
    ///
    /// ```text
    /// book-test/
    /// ├── book
    /// └── src
    ///     ├── chapter_1.md
    ///     └── SUMMARY.md
    /// ```
    ///
    /// It uses the path provided as the root directory for your book, then adds
    /// in a `src/` directory containing a `SUMMARY.md` and `chapter_1.md` file
    /// to get you started.
    pub fn init<P: Into<PathBuf>>(book_root: P) -> BookBuilder {
        BookBuilder::new(book_root)
    }

    /// Tells the renderer to build our book and put it in the build directory.
    pub fn build(&mut self) -> Result<()> {
        debug!("[fn]: build");

        let dest = self.get_destination();
        if dest.exists() {
            utils::fs::remove_dir_content(&dest).chain_err(|| "Unable to clear output directory")?;
        }

        self.renderer.render(self)
    }

    // FIXME: This doesn't belong as part of `MDBook`. It is only used by the HTML renderer
    #[doc(hidden)]
    pub fn write_file<P: AsRef<Path>>(&self, filename: P, content: &[u8]) -> Result<()> {
        let path = self.get_destination().join(filename);

        utils::fs::create_file(&path)?
            .write_all(content)
            .map_err(|e| e.into())
    }

    /// You can change the default renderer to another one by using this method.
    /// The only requirement is for your renderer to implement the [Renderer
    /// trait](../../renderer/renderer/trait.Renderer.html)
    pub fn set_renderer<R: Renderer + 'static>(mut self, renderer: R) -> Self {
        self.renderer = Box::new(renderer);
        self
    }

    /// Run `rustdoc` tests on the book, linking against the provided libraries.
    pub fn test(&mut self, library_paths: Vec<&str>) -> Result<()> {
        let library_args: Vec<&str> = (0..library_paths.len())
            .map(|_| "-L")
            .zip(library_paths.into_iter())
            .flat_map(|x| vec![x.0, x.1])
            .collect();

        let temp_dir = TempDir::new("mdbook")?;

        for item in self.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                if !ch.path.as_os_str().is_empty() {
                    let path = self.source_dir().join(&ch.path);
                    let base = path.parent()
                        .ok_or_else(|| String::from("Invalid bookitem path!"))?;
                    let content = utils::fs::file_to_string(&path)?;
                    // Parse and expand links
                    let content = preprocess::links::replace_all(&content, base)?;
                    println!("[*]: Testing file: {:?}", path);

                    // write preprocessed file to tempdir
                    let path = temp_dir.path().join(&ch.path);
                    let mut tmpf = utils::fs::create_file(&path)?;
                    tmpf.write_all(content.as_bytes())?;

                    let output = Command::new("rustdoc")
                        .arg(&path)
                        .arg("--test")
                        .args(&library_args)
                        .output()?;

                    if !output.status.success() {
                        bail!(ErrorKind::Subprocess(
                            "Rustdoc returned an error".to_string(),
                            output
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    // FIXME: This doesn't belong under `MDBook`, it should really be passed to the renderer directly.
    #[doc(hidden)]
    pub fn get_destination(&self) -> PathBuf {
        self.root.join(&self.config.build.build_dir)
    }

    /// Get the directory containing this book's source files.
    pub fn source_dir(&self) -> PathBuf {
        self.root.join(&self.config.book.src)
    }

    // FIXME: This belongs as part of the `HtmlConfig`.
    #[doc(hidden)]
    pub fn theme_dir(&self) -> PathBuf {
        match self.config.html_config().and_then(|h| h.theme) {
            Some(d) => self.root.join(d),
            None => self.root.join("theme"),
        }
    }
}
