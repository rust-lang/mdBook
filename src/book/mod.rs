pub mod bookitem;

pub use self::bookitem::{BookItem, BookItems};

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use tempdir::TempDir;

use {parse, theme, utils};
use renderer::{HtmlHandlebars, Renderer};
use preprocess;
use errors::*;

use config::Config;

pub struct MDBook {
    pub root: PathBuf,
    pub config: Config,

    pub content: Vec<BookItem>,
    renderer: Box<Renderer>,

    pub livereload: Option<String>,
}

impl MDBook {
    /// Create a new `MDBook` struct with root directory `root`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate mdbook;
    /// # use mdbook::MDBook;
    /// # #[allow(unused_variables)]
    /// # fn main() {
    /// let book = MDBook::new("root_dir");
    /// # }
    /// ```
    ///
    /// In this example, `root_dir` will be the root directory of our book
    /// and is specified in function of the current working directory
    /// by using a relative path instead of an
    /// absolute path.
    ///
    /// Default directory paths:
    ///
    /// - source: `root/src`
    /// - output: `root/book`
    /// - theme: `root/theme`
    ///
    /// They can both be changed by using [`set_src()`](#method.set_src) and
    /// [`set_dest()`](#method.set_dest)

    pub fn new<P: Into<PathBuf>>(root: P) -> MDBook {
        let root = root.into();
        if !root.exists() || !root.is_dir() {
            warn!("{:?} No directory with that name", root);
        }

        MDBook {
            root: root,
            config: Config::default(),

            content: vec![],
            renderer: Box::new(HtmlHandlebars::new()),

            livereload: None,
        }
    }

    /// Returns a flat depth-first iterator over the elements of the book,
    /// it returns an [BookItem enum](bookitem.html):
    /// `(section: String, bookitem: &BookItem)`
    ///
    /// ```no_run
    /// # extern crate mdbook;
    /// # use mdbook::MDBook;
    /// # use mdbook::BookItem;
    /// # #[allow(unused_variables)]
    /// # fn main() {
    /// # let book = MDBook::new("mybook");
    /// for item in book.iter() {
    ///     match item {
    ///         &BookItem::Chapter(ref section, ref chapter) => {},
    ///         &BookItem::Affix(ref chapter) => {},
    ///         &BookItem::Spacer => {},
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
        BookItems {
            items: &self.content[..],
            current_index: 0,
            stack: Vec::new(),
        }
    }

    /// `init()` creates some boilerplate files and directories
    /// to get you started with your book.
    ///
    /// ```text
    /// book-test/
    /// ├── book
    /// └── src
    ///     ├── chapter_1.md
    ///     └── SUMMARY.md
    /// ```
    ///
    /// It uses the paths given as source and output directories
    /// and adds a `SUMMARY.md` and a
    /// `chapter_1.md` to the source directory.

    pub fn init(&mut self) -> Result<()> {
        debug!("[fn]: init");

        if !self.root.exists() {
            fs::create_dir_all(&self.root).unwrap();
            info!("{:?} created", self.root.display());
        }

        {
            let dest = self.get_destination();
            if !dest.exists() {
                debug!("[*]: {} does not exist, trying to create directory", dest.display());
                fs::create_dir_all(dest)?;
            }


            let src = self.get_source();
            if !src.exists() {
                debug!("[*]: {} does not exist, trying to create directory", src.display());
                fs::create_dir_all(&src)?;
            }

            let summary = src.join("SUMMARY.md");

            if !summary.exists() {
                // Summary does not exist, create it
                debug!("[*]: {:?} does not exist, trying to create SUMMARY.md",
                       &summary);
                let mut f = File::create(&summary)?;

                debug!("[*]: Writing to SUMMARY.md");

                writeln!(f, "# Summary")?;
                writeln!(f, "")?;
                writeln!(f, "- [Chapter 1](./chapter_1.md)")?;
            }
        }

        // parse SUMMARY.md, and create the missing item related file
        self.parse_summary()?;

        debug!("[*]: constructing paths for missing files");
        for item in self.iter() {
            debug!("[*]: item: {:?}", item);
            let ch = match *item {
                BookItem::Spacer => continue,
                BookItem::Chapter(_, ref ch) | BookItem::Affix(ref ch) => ch,
            };
            if !ch.path.as_os_str().is_empty() {
                let path = self.get_source().join(&ch.path);

                if !path.exists() {
                    if !self.config.build.create_missing {
                        return Err(
                            format!("'{}' referenced from SUMMARY.md does not exist.", path.to_string_lossy()).into(),
                        );
                    }
                    debug!("[*]: {:?} does not exist, trying to create file", path);
                    ::std::fs::create_dir_all(path.parent().unwrap())?;
                    let mut f = File::create(path)?;

                    // debug!("[*]: Writing to {:?}", path);
                    writeln!(f, "# {}", ch.name)?;
                }
            }
        }

        debug!("[*]: init done");
        Ok(())
    }

    pub fn create_gitignore(&self) {
        let gitignore = self.get_gitignore();

        let destination = self.get_destination();

        // Check that the gitignore does not extist and that the destination path
        // begins with the root path
        // We assume tha if it does begin with the root path it is contained within.
        // This assumption
        // will not hold true for paths containing double dots to go back up e.g.
        // `root/../destination`
        if !gitignore.exists() && destination.starts_with(&self.root) {
            let relative = destination
                .strip_prefix(&self.root)
                .expect("Could not strip the root prefix, path is not relative to root")
                .to_str()
                .expect("Could not convert to &str");

            debug!("[*]: {:?} does not exist, trying to create .gitignore", gitignore);

            let mut f = File::create(&gitignore).expect("Could not create file.");

            debug!("[*]: Writing to .gitignore");

            writeln!(f, "/{}", relative).expect("Could not write to file.");
        }
    }

    /// The `build()` method is the one where everything happens.
    /// First it parses `SUMMARY.md` to construct the book's structure
    /// in the form of a `Vec<BookItem>` and then calls `render()`
    /// method of the current renderer.
    ///
    /// It is the renderer who generates all the output files.
    pub fn build(&mut self) -> Result<()> {
        debug!("[fn]: build");

        self.init()?;

        // Clean output directory
        utils::fs::remove_dir_content(&self.get_destination())?;

        self.renderer.render(self)
    }


    pub fn get_gitignore(&self) -> PathBuf {
        self.root.join(".gitignore")
    }

    pub fn copy_theme(&self) -> Result<()> {
        debug!("[fn]: copy_theme");

        let themedir = self.theme_dir();

        if !themedir.exists() {
            debug!("[*]: {:?} does not exist, trying to create directory",
                   themedir);
            fs::create_dir(&themedir)?;
        }

        // index.hbs
        let mut index = File::create(themedir.join("index.hbs"))?;
        index.write_all(theme::INDEX)?;

        // header.hbs
        let mut header = File::create(themedir.join("header.hbs"))?;
        header.write_all(theme::HEADER)?;

        // book.css
        let mut css = File::create(themedir.join("book.css"))?;
        css.write_all(theme::CSS)?;

        // favicon.png
        let mut favicon = File::create(themedir.join("favicon.png"))?;
        favicon.write_all(theme::FAVICON)?;

        // book.js
        let mut js = File::create(themedir.join("book.js"))?;
        js.write_all(theme::JS)?;

        // highlight.css
        let mut highlight_css = File::create(themedir.join("highlight.css"))?;
        highlight_css.write_all(theme::HIGHLIGHT_CSS)?;

        // highlight.js
        let mut highlight_js = File::create(themedir.join("highlight.js"))?;
        highlight_js.write_all(theme::HIGHLIGHT_JS)?;

        Ok(())
    }

    pub fn write_file<P: AsRef<Path>>(&self, filename: P, content: &[u8]) -> Result<()> {
        let path = self.get_destination().join(filename);

        utils::fs::create_file(&path)?.write_all(content)
                                      .map_err(|e| e.into())
    }

    /// Parses the `book.json` file (if it exists) to extract
    /// the configuration parameters.
    /// The `book.json` file should be in the root directory of the book.
    /// The root directory is the one specified when creating a new `MDBook`

    pub fn read_config(mut self) -> Result<Self> {
        let config_path = self.root.join("book.toml");
        debug!("[*] Loading the config from {}", config_path.display());
        self.config = Config::from_disk(&config_path)?;

        Ok(self)
    }

    /// You can change the default renderer to another one
    /// by using this method. The only requirement
    /// is for your renderer to implement the
    /// [Renderer trait](../../renderer/renderer/trait.Renderer.html)
    ///
    /// ```no_run
    /// extern crate mdbook;
    /// use mdbook::MDBook;
    /// use mdbook::renderer::HtmlHandlebars;
    ///
    /// # #[allow(unused_variables)]
    /// fn main() {
    ///     let book = MDBook::new("mybook")
    ///                         .set_renderer(Box::new(HtmlHandlebars::new()));
    ///
    /// // In this example we replace the default renderer
    /// // by the default renderer...
    /// // Don't forget to put your renderer in a Box
    /// }
    /// ```
    ///
    /// **note:** Don't forget to put your renderer in a `Box`
    /// before passing it to `set_renderer()`

    pub fn set_renderer(mut self, renderer: Box<Renderer>) -> Self {
        self.renderer = renderer;
        self
    }

    pub fn test(&mut self, library_paths: Vec<&str>) -> Result<()> {
        // read in the chapters
        self.parse_summary().chain_err(|| "Couldn't parse summary")?;
        let library_args: Vec<&str> = (0..library_paths.len())
            .map(|_| "-L")
            .zip(library_paths.into_iter())
            .flat_map(|x| vec![x.0, x.1])
            .collect();
        let temp_dir = TempDir::new("mdbook")?;
        for item in self.iter() {
            if let BookItem::Chapter(_, ref ch) = *item {
                if !ch.path.as_os_str().is_empty() {
                    let path = self.get_source().join(&ch.path);
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

                    let output = Command::new("rustdoc").arg(&path)
                                                        .arg("--test")
                                                        .args(&library_args)
                                                        .output()?;

                    if !output.status.success() {
                        bail!(ErrorKind::Subprocess("Rustdoc returned an error".to_string(),
                                                    output));
                    }
                }
            }
        }
        Ok(())
    }

    // Construct book
    fn parse_summary(&mut self) -> Result<()> {
        // When append becomes stable, use self.content.append() ...
        let summary = self.get_source().join("SUMMARY.md");
        self.content = parse::construct_bookitems(&summary)?;
        Ok(())
    }

    pub fn get_destination(&self) -> PathBuf {
        self.root.join(&self.config.build.build_dir)
    }

    pub fn get_source(&self) -> PathBuf {
        self.root.join(&self.config.book.src)
    }

    pub fn theme_dir(&self) -> PathBuf {
        match self.config.html_config().and_then(|h| h.theme) {
            Some(d) => self.root.join(d),
            None => self.root.join("theme"),
        }
    }
}
