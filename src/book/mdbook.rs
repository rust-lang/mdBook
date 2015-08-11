use std::path::{Path, PathBuf};
use std::fs::{self, File, metadata};
use std::io::Write;
use std::error::Error;

use {BookConfig, BookItem};
use book::BookItems;
use parse;
use theme;
use renderer::Renderer;
use renderer::HtmlHandlebars;

pub struct MDBook {
    config: BookConfig,
    pub root: PathBuf,
    pub content: Vec<BookItem>,
    renderer: Box<Renderer>,
}

impl MDBook {

    /// Create a new `MDBook` struct with root directory `root`
    ///
    /// - The default source directory is set to `root/src`
    /// - The default output directory is set to `root/book`
    ///
    /// They can both be changed by using [`set_src()`](#method.set_src) and [`set_dest()`](#method.set_dest)

    pub fn new(root: &Path) -> MDBook {

        // Hacky way to check if the path exists... Until PathExt moves to stable
        match metadata(root) {
            Err(_) => panic!("Directory does not exist"),
            Ok(f) => {
                if !f.is_dir() {
                    panic!("Is not a directory");
                }
            }
        }

        MDBook {
            root: root.to_path_buf(),
            content: vec![],
            config: BookConfig::new()
                        .set_src(&root.join("src"))
                        .set_dest(&root.join("book"))
                        .to_owned(),
            renderer: Box::new(HtmlHandlebars::new()),
        }
    }

    /// Returns a flat depth-first iterator over the elements of the book in the form of a tuple:
    /// `(section: String, bookitem: &BookItem)`
    ///
    /// ```no_run
    /// # extern crate mdbook;
    /// # use mdbook::MDBook;
    /// # use std::path::Path;
    /// # fn main() {
    /// # let mut book = MDBook::new(Path::new("mybook"));
    /// for (section, element) in book.iter() {
    ///     println!("{} {}", section, element.name);
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

    /// `init()` creates some boilerplate files and directories to get you started with your book.
    ///
    /// ```text
    /// book-test/
    /// ├── book
    /// └── src
    ///     ├── chapter_1.md
    ///     └── SUMMARY.md
    /// ```
    ///
    /// It uses the paths given as source and output directories and adds a `SUMMARY.md` and a
    /// `chapter_1.md` to the source directory.

    pub fn init(&self) -> Result<(), Box<Error>> {

        debug!("[fn]: init");

        let dest = self.config.get_dest();
        let src = self.config.get_src();

        // Hacky way to check if the directory exists... Until PathExt moves to stable
        match metadata(&dest) {
            Err(_) => {
                // There is a very high chance that the error is due to the fact that
                // the directory / file does not exist
                debug!("[*]: {:?} does not exist, trying to create directory", dest);
                fs::create_dir(&dest).unwrap();
            },
            Ok(_) => { /* If there is no error, the directory / file does exist */ }
        }

        // Hacky way to check if the directory exists... Until PathExt moves to stable
        match metadata(&src) {
            Err(_) => {
                // There is a very high chance that the error is due to the fact that
                // the directory / file does not exist
                debug!("[*]: {:?} does not exist, trying to create directory", src);
                fs::create_dir(&src).unwrap();
            },
            Ok(_) => { /* If there is no error, the directory / file does exist */ }
        }

        // Hacky way to check if the directory exists... Until PathExt moves to stable
        let summary = match metadata(&src.join("SUMMARY.md")) {
            Err(_) => {
                // There is a very high chance that the error is due to the fact that
                // the directory / file does not exist
                debug!("[*]: {:?} does not exist, trying to create SUMMARY.md", src.join("SUMMARY.md"));
                Ok(File::create(&src.join("SUMMARY.md")).unwrap())
            },
            Ok(_) => {
                /* If there is no error, the directory / file does exist */
                Err("SUMMARY.md does already exist")
            }
        };

        if let Ok(mut f) = summary {
            debug!("[*]: Writing to SUMMARY.md");

            try!(writeln!(f, "# Summary"));
            try!(writeln!(f, ""));
            try!(writeln!(f, "- [Chapter 1](./chapter_1.md)"));

            let mut chapter_1 = File::create(&src.join("chapter_1.md")).unwrap();
            try!(writeln!(chapter_1, "# Chapter 1"));
        }

        return Ok(());
    }

    /// The `build()` method is the one where everything happens. First it parses `SUMMARY.md` to
    /// construct the book's structure in the form of a `Vec<BookItem>` and then calls `render()`
    /// method of the current renderer.
    ///
    /// It is the renderer who generates all the output files.

    pub fn build(&mut self) -> Result<(), Box<Error>> {
        debug!("[fn]: build");

        try!(self.parse_summary());

        try!(self.renderer.render(
            self.iter(),
            &self.config,
        ));

        Ok(())
    }


    pub fn copy_theme(&self) -> Result<(), Box<Error>> {
        debug!("[fn]: copy_theme");

        let theme_dir = self.config.get_src().join("theme");

        // Hacky way to check if the directory exists... Until PathExt moves to stable
        match metadata(&theme_dir) {
            Err(_) => {
                // There is a very high chance that the error is due to the fact that
                // the directory / file does not exist
                debug!("[*]: {:?} does not exist, trying to create directory", theme_dir);
                fs::create_dir(&theme_dir).unwrap();
            },
            Ok(_) => { /* If there is no error, the directory / file does exist */ }
        }

        // index.hbs
        let mut index = try!(File::create(&theme_dir.join("index.hbs")));
        try!(index.write_all(theme::INDEX));

        // book.css
        let mut css = try!(File::create(&theme_dir.join("book.css")));
        try!(css.write_all(theme::CSS));

        // book.js
        let mut js = try!(File::create(&theme_dir.join("book.js")));
        try!(js.write_all(theme::JS));

        // highlight.css
        let mut highlight_css = try!(File::create(&theme_dir.join("highlight.css")));
        try!(highlight_css.write_all(theme::HIGHLIGHT_CSS));

        // highlight.js
        let mut highlight_js = try!(File::create(&theme_dir.join("highlight.js")));
        try!(highlight_js.write_all(theme::HIGHLIGHT_JS));

        Ok(())
    }

    /// Parses the `book.json` file (if it exists) to extract the configuration parameters.
    /// The `book.json` file should be in the root directory of the book.
    /// The root directory is the one specified when creating a new `MDBook`
    ///
    /// ```no_run
    /// # extern crate mdbook;
    /// # use mdbook::MDBook;
    /// # use std::path::Path;
    /// # fn main() {
    /// let mut book = MDBook::new(Path::new("root_dir"));
    /// # }
    /// ```
    ///
    /// In this example, `root_dir` will be the root directory of our book and is specified in function
    /// of the current working directory by using a relative path instead of an absolute path.

    pub fn read_config(mut self) -> Self {
        self.config.read_config(&self.root);
        self
    }

    /// You can change the default renderer to another one by using this method. The only requirement
    /// is for your renderer to implement the [Renderer trait](../../renderer/renderer/trait.Renderer.html)
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
    ///     // In this example we replace the default renderer by the default renderer...
    ///     // Don't forget to put your renderer in a Box
    /// }
    /// ```
    ///
    /// **note:** Don't forget to put your renderer in a `Box` before passing it to `set_renderer()`

    pub fn set_renderer(mut self, renderer: Box<Renderer>) -> Self {
        self.renderer = renderer;
        self
    }

    pub fn set_dest(mut self, dest: &Path) -> Self {
        self.config.set_dest(&self.root.join(dest));
        self
    }

    pub fn get_dest(&self) -> &Path {
        self.config.get_dest()
    }

    pub fn set_src(mut self, src: &Path) -> Self {
        self.config.set_src(&self.root.join(src));
        self
    }

    pub fn get_src(&self) -> &Path {
        self.config.get_src()
    }

    pub fn set_title(mut self, title: &str) -> Self {
        self.config.title = title.to_owned();
        self
    }

    pub fn set_author(mut self, author: &str) -> Self {
        self.config.author = author.to_owned();
        self
    }


    // Construct book
    fn parse_summary(&mut self) -> Result<(), Box<Error>> {

        // When append becomes stable, use self.content.append() ...
        let book_items = try!(parse::construct_bookitems(&self.config.get_src().join("SUMMARY.md")));

        for item in book_items {
            self.content.push(item)
        }

        Ok(())
    }

}
