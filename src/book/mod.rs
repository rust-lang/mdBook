//! The internal representation of a `Book`.

mod book;
mod summary;

pub use self::book::{load_book, Book, BookItem, BookItems, Chapter};
pub use self::summary::SectionNumber;

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::Command;
use tempdir::TempDir;

use {theme, utils};
use renderer::{Renderer, HtmlHandlebars};
use preprocess;
use errors::*;

use config::BookConfig;
use config::tomlconfig::TomlConfig;
use config::htmlconfig::HtmlConfig;
use config::jsonconfig::JsonConfig;


/// A helper for managing the `Book`, its configuration, and the rendering 
/// process.
pub struct MDBook {
    config: BookConfig,

    pub content: Option<Book>,
    renderer: Box<Renderer>,

    livereload: Option<String>,

    /// Should `mdbook build` create files referenced from SUMMARY.md if they
    /// don't exist
    pub create_missing: bool,
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
            config: BookConfig::new(root),

            content: None,
            renderer: Box::new(HtmlHandlebars::new()),

            livereload: None,
            create_missing: true,
        }
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
    /// # let book = MDBook::new("mybook");
    /// for item in book.iter() {
    ///     match *item {
    ///         BookItem::Chapter(ref chapter) => println!("{}", chapter),
    ///         BookItem::Separator => {},
    ///     }
    /// }
    /// panic!();
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
        self.content.as_ref().expect("Trying to iterate over a book before it is loaded. This is a bug")
        .iter()
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

        {
            let root = self.config.get_root();
            let dest = self.get_destination();
            let src = self.config.get_source();

            let necessary_folders = &[root, dest, src];
            
            for folder in necessary_folders {
                if !folder.exists() {
                    fs::create_dir_all(folder)?;
                    debug!("{} created", folder.display());
                }
            }

            let summary = src.join("SUMMARY.md");

            if !summary.exists() {
                debug!("[*]: Creating SUMMARY.md");

                let mut f = File::create(&summary)?;

                writeln!(f, "# Summary")?;
                writeln!(f)?;
                writeln!(f, "- [Chapter 1](./chapter_1.md)")?;
            }

            let ch_1 = src.join("chapter_1.md");
            if !ch_1.exists() {
                debug!("[*] Creating {}", ch_1.display());

                let mut f = File::create(&ch_1)?;
                writeln!(f, "# Chapter 1")?;
            }
        }

        // parse SUMMARY.md and load the newly created files into memory
        self.parse_summary().chain_err(|| "Couldn't parse the SUMMARY.md file")?;

        debug!("[*]: init done");
        Ok(())
    }

    pub fn create_gitignore(&self) {
        let gitignore = self.get_gitignore();

        let destination = self.config.get_html_config()
                                     .get_destination();

        // Check that the gitignore does not extist and that the destination path begins with the root path
        // We assume tha if it does begin with the root path it is contained within. This assumption
        // will not hold true for paths containing double dots to go back up e.g. `root/../destination`
        if !gitignore.exists() && destination.starts_with(self.config.get_root()) {

            let relative = destination
                .strip_prefix(self.config.get_root())
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
        utils::fs::remove_dir_content(self.config.get_html_config().get_destination())?;

        self.renderer.render(self)
    }


    pub fn get_gitignore(&self) -> PathBuf {
        self.config.get_root().join(".gitignore")
    }

    pub fn copy_theme(&self) -> Result<()> {
        debug!("[fn]: copy_theme");

        let themedir = self.config.get_html_config().get_theme();
        if !themedir.exists() {
            debug!("[*]: {:?} does not exist, trying to create directory", themedir);
            fs::create_dir(&themedir)?;
        }

        // index.hbs
        let mut index = File::create(&themedir.join("index.hbs"))?;
        index.write_all(theme::INDEX)?;

        // book.css
        let mut css = File::create(&themedir.join("book.css"))?;
        css.write_all(theme::CSS)?;

        // favicon.png
        let mut favicon = File::create(&themedir.join("favicon.png"))?;
        favicon.write_all(theme::FAVICON)?;

        // book.js
        let mut js = File::create(&themedir.join("book.js"))?;
        js.write_all(theme::JS)?;

        // highlight.css
        let mut highlight_css = File::create(&themedir.join("highlight.css"))?;
        highlight_css.write_all(theme::HIGHLIGHT_CSS)?;

        // highlight.js
        let mut highlight_js = File::create(&themedir.join("highlight.js"))?;
        highlight_js.write_all(theme::HIGHLIGHT_JS)?;

        Ok(())
    }

    pub fn write_file<P: AsRef<Path>>(&self, filename: P, content: &[u8]) -> Result<()> {
        let path = self.get_destination()
            .join(filename);

        utils::fs::create_file(&path)?
            .write_all(content)
            .map_err(|e| e.into())
    }

    /// Parses the `book.json` file (if it exists) to extract
    /// the configuration parameters.
    /// The `book.json` file should be in the root directory of the book.
    /// The root directory is the one specified when creating a new `MDBook`

    pub fn read_config(mut self) -> Result<Self> {

        let toml = self.get_root().join("book.toml");
        let json = self.get_root().join("book.json");

        if toml.exists() {
            let mut file = File::open(toml)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let parsed_config = TomlConfig::from_toml(&content)?;
            self.config.fill_from_tomlconfig(parsed_config);
        } else if json.exists() {
            warn!("The JSON configuration file is deprecated, please use the TOML configuration.");
            let mut file = File::open(json)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let parsed_config = JsonConfig::from_json(&content)?;
            self.config.fill_from_jsonconfig(parsed_config);
        }

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
        let library_args: Vec<&str> = (0..library_paths.len()).map(|_| "-L")
                                                              .zip(library_paths.into_iter())
                                                              .flat_map(|x| vec![x.0, x.1])
                                                              .collect();
        let temp_dir = TempDir::new("mdbook")?;
        for item in self.iter() {

            if let BookItem::Chapter(ref ch) = *item {
                if ch.path != PathBuf::new() {

                    let path = self.get_source().join(&ch.path);
                    let base = path.parent().ok_or_else(
                        || String::from("Invalid bookitem path!"),
                    )?;
                    let content = utils::fs::file_to_string(&path)?;
                    // Parse and expand links
                    let content = preprocess::links::replace_all(&content, base)?;
                    println!("[*]: Testing file: {:?}", path);

                    //write preprocessed file to tempdir
                    let path = temp_dir.path().join(&ch.path);
                    let mut tmpf = utils::fs::create_file(&path)?;
                    tmpf.write_all(content.as_bytes())?;

                    let output = Command::new("rustdoc").arg(&path).arg("--test").args(&library_args).output()?;

                    if !output.status.success() {
                        bail!(ErrorKind::Subprocess("Rustdoc returned an error".to_string(), output));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_root(&self) -> &Path {
        self.config.get_root()
    }


    pub fn with_destination<T: Into<PathBuf>>(mut self, destination: T) -> Self {
        let root = self.config.get_root().to_owned();
        self.config.get_mut_html_config()
            .set_destination(&root, &destination.into());
        self
    }


    pub fn get_destination(&self) -> &Path {
        self.config.get_html_config()
            .get_destination()
    }

    pub fn with_source<T: Into<PathBuf>>(mut self, source: T) -> Self {
        self.config.set_source(source);
        self
    }

    pub fn get_source(&self) -> &Path {
        self.config.get_source()
    }

    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.config.set_title(title);
        self
    }

    pub fn get_title(&self) -> &str {
        self.config.get_title()
    }

    pub fn with_description<T: Into<String>>(mut self, description: T) -> Self {
        self.config.set_description(description);
        self
    }

    pub fn get_description(&self) -> &str {
        self.config.get_description()
    }

    pub fn set_livereload(&mut self, livereload: String) -> &mut Self {
        self.livereload = Some(livereload);
        self
    }

    pub fn unset_livereload(&mut self) -> &Self {
        self.livereload = None;
        self
    }

    pub fn get_livereload(&self) -> Option<&String> {
        self.livereload.as_ref()
    }

    pub fn with_theme_path<T: Into<PathBuf>>(mut self, theme_path: T) -> Self {
        let root = self.config.get_root().to_owned();
        self.config.get_mut_html_config()
            .set_theme(&root, &theme_path.into());
        self
    }

    pub fn get_theme_path(&self) -> &Path {
        self.config.get_html_config()
            .get_theme()
    }

    pub fn with_curly_quotes(mut self, curly_quotes: bool) -> Self {
        self.config.get_mut_html_config()
            .set_curly_quotes(curly_quotes);
        self
    }

    pub fn get_curly_quotes(&self) -> bool {
        self.config.get_html_config()
            .get_curly_quotes()
    }

    pub fn with_mathjax_support(mut self, mathjax_support: bool) -> Self {
        self.config.get_mut_html_config()
            .set_mathjax_support(mathjax_support);
        self
    }

    pub fn get_mathjax_support(&self) -> bool {
        self.config.get_html_config()
            .get_mathjax_support()
    }

    pub fn get_google_analytics_id(&self) -> Option<String> {
        self.config.get_html_config()
            .get_google_analytics_id()
    }

    pub fn has_additional_js(&self) -> bool {
        self.config.get_html_config()
            .has_additional_js()
    }

    pub fn get_additional_js(&self) -> &[PathBuf] {
        self.config.get_html_config()
            .get_additional_js()
    }

    pub fn has_additional_css(&self) -> bool {
        self.config.get_html_config()
            .has_additional_css()
    }

    pub fn get_additional_css(&self) -> &[PathBuf] {
        self.config.get_html_config()
            .get_additional_css()
    }

    pub fn get_html_config(&self) -> &HtmlConfig {
        self.config.get_html_config()
    }

    // Construct book
    fn parse_summary(&mut self) -> Result<()> {
        let src = self.config.get_source();
        let book = load_book(&src)?;

        self.content = Some(book);
        Ok(())
    }
}
