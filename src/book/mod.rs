pub mod bookitem;

pub use self::bookitem::{BookItem, BookItems};

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::error::Error;
use std::io;
use std::io::{Read, Write};
use std::io::ErrorKind;
use std::process::Command;

use {theme, parse, utils};
use renderer::{Renderer, HtmlHandlebars};

use config::{BookConfig, HtmlConfig};
use config::tomlconfig::TomlConfig;
use config::jsonconfig::JsonConfig;


pub struct MDBook {
    config: BookConfig,

    pub content: Vec<BookItem>,
    renderer: Box<Renderer>,

    livereload: Option<String>,

    /// Should `mdbook build` create files referenced from SUMMARY.md if they
    /// don't exist
    pub create_missing: bool,

    pub google_analytics: Option<String>,
}

impl MDBook {
    /// Create a new `MDBook` struct with root directory `root`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate mdbook;
    /// # use mdbook::MDBook;
    /// # use std::path::Path;
    /// # fn main() {
    /// let book = MDBook::new(Path::new("root_dir"));
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

    pub fn new(root: &Path) -> MDBook {

        if !root.exists() || !root.is_dir() {
            warn!("{:?} No directory with that name", root);
        }

        MDBook {
            config: BookConfig::new(root),

            content: vec![],
            renderer: Box::new(HtmlHandlebars::new()),

            livereload: None,
            create_missing: true,
            google_analytics: None,
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
    /// # use std::path::Path;
    /// # fn main() {
    /// # let mut book = MDBook::new(Path::new("mybook"));
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

    pub fn init(&mut self) -> Result<(), Box<Error>> {

        debug!("[fn]: init");

        if !self.config.get_root().exists() {
            fs::create_dir_all(&self.config.get_root()).unwrap();
            info!("{:?} created", &self.config.get_root());
        }

        {

            if let Some(htmlconfig) = self.config.get_html_config() {
                if !htmlconfig.get_destination().exists() {
                    debug!("[*]: {:?} does not exist, trying to create directory", htmlconfig.get_destination());
                    fs::create_dir_all(htmlconfig.get_destination())?;
                }
            }
            

            if !self.config.get_source().exists() {
                debug!("[*]: {:?} does not exist, trying to create directory", self.config.get_source());
                fs::create_dir_all(self.config.get_source())?;
            }

            let summary = self.config.get_source().join("SUMMARY.md");

            if !summary.exists() {

                // Summary does not exist, create it
                debug!("[*]: {:?} does not exist, trying to create SUMMARY.md", &summary);
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
                BookItem::Chapter(_, ref ch) |
                BookItem::Affix(ref ch) => ch,
            };
            if !ch.path.as_os_str().is_empty() {
                let path = self.config.get_source().join(&ch.path);

                if !path.exists() {
                    if !self.create_missing {
                        return Err(format!("'{}' referenced from SUMMARY.md does not exist.", path.to_string_lossy())
                                       .into());
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

        // If the HTML renderer is not set, return
        if self.config.get_html_config().is_none() { return; }
        
        let destination = self.config.get_html_config()
                                     .expect("The HtmlConfig does exist, checked just before")
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

            writeln!(f, "{}", relative).expect("Could not write to file.");
        }
    }

    /// The `build()` method is the one where everything happens.
    /// First it parses `SUMMARY.md` to construct the book's structure
    /// in the form of a `Vec<BookItem>` and then calls `render()`
    /// method of the current renderer.
    ///
    /// It is the renderer who generates all the output files.
    pub fn build(&mut self) -> Result<(), Box<Error>> {
        debug!("[fn]: build");

        self.init()?;

        // Clean output directory
        if let Some(htmlconfig) = self.config.get_html_config() {
            utils::fs::remove_dir_content(htmlconfig.get_destination())?;
        }
        
        self.renderer.render(&self)?;

        Ok(())
    }


    pub fn get_gitignore(&self) -> PathBuf {
        self.config.get_root().join(".gitignore")
    }

    pub fn copy_theme(&self) -> Result<(), Box<Error>> {
        debug!("[fn]: copy_theme");

        if let Some(themedir) = self.config.get_html_config().and_then(HtmlConfig::get_theme) {

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
        }
        
        Ok(())
    }

    pub fn write_file<P: AsRef<Path>>(&self, filename: P, content: &[u8]) -> Result<(), Box<Error>> {
        let path = self.get_destination()
            .ok_or(String::from("HtmlConfig not set, could not find a destination"))?
            .join(filename);

        utils::fs::create_file(&path)
            .and_then(|mut file| file.write_all(content))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Could not create {}: {}", path.display(), e)))?;

        Ok(())
    }

    /// Parses the `book.json` file (if it exists) to extract
    /// the configuration parameters.
    /// The `book.json` file should be in the root directory of the book.
    /// The root directory is the one specified when creating a new `MDBook`

    pub fn read_config(mut self) -> Result<Self, Box<Error>> {

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
    /// # use std::path::Path;
    ///
    /// fn main() {
    ///     let mut book = MDBook::new(Path::new("mybook"))
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

    pub fn test(&mut self) -> Result<(), Box<Error>> {
        // read in the chapters
        self.parse_summary()?;
        for item in self.iter() {

            if let BookItem::Chapter(_, ref ch) = *item {
                if ch.path != PathBuf::new() {

                    let path = self.get_source().join(&ch.path);

                    println!("[*]: Testing file: {:?}", path);

                    let output_result = Command::new("rustdoc").arg(&path).arg("--test").output();
                    let output = output_result?;

                    if !output.status.success() {
                        return Err(Box::new(io::Error::new(ErrorKind::Other,
                                                           format!("{}\n{}",
                                                                   String::from_utf8_lossy(&output.stdout),
                                                                   String::from_utf8_lossy(&output.stderr)))) as
                                   Box<Error>);
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
        if let Some(htmlconfig) = self.config.get_mut_html_config() {
            htmlconfig.set_destination(&root, &destination.into());
        } else {
            error!("There is no HTML renderer set...");
        }

        self
    }
    

    pub fn get_destination(&self) -> Option<&Path> {
        if let Some(htmlconfig) = self.config.get_html_config() {
            return Some(htmlconfig.get_destination());
        }

        None
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

/*
    pub fn set_author(mut self, author: &str) -> Self {
        self.author = author.to_owned();
        self
    }

    pub fn get_author(&self) -> &str {
        &self.author
    }
*/
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
        if let Some(htmlconfig) = self.config.get_mut_html_config() {
            htmlconfig.set_theme(&root, &theme_path.into());
        } else {
            error!("There is no HTML renderer set...");
        }
        self
    }

    pub fn get_theme_path(&self) -> Option<&PathBuf> {
        if let Some(htmlconfig) = self.config.get_html_config() {
            return htmlconfig.get_theme();
        }

        None
    }

    // Construct book
    fn parse_summary(&mut self) -> Result<(), Box<Error>> {
        // When append becomes stable, use self.content.append() ...
        self.content = parse::construct_bookitems(&self.get_source().join("SUMMARY.md"))?;
        Ok(())
    }
}
