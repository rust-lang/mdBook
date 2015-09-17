use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use std::error::Error;

use {BookConfig, BookItem, theme, parse, utils};
use book::BookItems;
use renderer::{Renderer, HtmlHandlebars};
use utils::{PathExt, create_path};

pub struct MDBook {
    config: BookConfig,
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

        if !root.exists() || !root.is_dir() {
            output!("{:?} No directory with that name", root);
        }

        MDBook {
            content: vec![],
            config: BookConfig::new(root)
                        .set_src(&root.join("src"))
                        .set_dest(&root.join("book"))
                        .to_owned(),
            renderer: Box::new(HtmlHandlebars::new()),
        }
    }

    /// Returns a flat depth-first iterator over the elements of the book, it returns an [BookItem enum](bookitem.html):
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

    pub fn init(&mut self) -> Result<(), Box<Error>> {

        debug!("[fn]: init");

        if !self.config.get_root().exists() {
            create_path(self.config.get_root()).unwrap();
            output!("{:?} created", self.config.get_root());
        }

        {
            let dest = self.config.get_dest();
            let src = self.config.get_src();

            if !dest.exists() {
                debug!("[*]: {:?} does not exist, trying to create directory", dest);
                try!(fs::create_dir(&dest));
            }

            if !src.exists() {
                debug!("[*]: {:?} does not exist, trying to create directory", src);
                try!(fs::create_dir(&src));
            }

            let summary = src.join("SUMMARY.md");

            if !summary.exists() {

                // Summary does not exist, create it

                debug!("[*]: {:?} does not exist, trying to create SUMMARY.md", src.join("SUMMARY.md"));
                let mut f = try!(File::create(&src.join("SUMMARY.md")));

                debug!("[*]: Writing to SUMMARY.md");

                try!(writeln!(f, "# Summary"));
                try!(writeln!(f, ""));
                try!(writeln!(f, "- [Chapter 1](./chapter_1.md)"));
            }
        }

        // parse SUMMARY.md, and create the missing item related file
        try!(self.parse_summary());

        debug!("[*]: constructing paths for missing files");
        for item in self.iter() {
            debug!("[*]: item: {:?}", item);
            match *item {
                BookItem::Spacer => continue,
                BookItem::Chapter(_, ref ch) | BookItem::Affix(ref ch) => {
                    if ch.path != PathBuf::new() {
                        let path = self.config.get_src().join(&ch.path);

                        if !path.exists() {
                            debug!("[*]: {:?} does not exist, trying to create file", path);
                            try!(::std::fs::create_dir_all(path.parent().unwrap()));
                            let mut f = try!(File::create(path));

                            //debug!("[*]: Writing to {:?}", path);
                            try!(writeln!(f, "# {}", ch.name));
                        }
                    }
                }
            }
        }

        debug!("[*]: init done");
        Ok(())
    }

    /// The `build()` method is the one where everything happens. First it parses `SUMMARY.md` to
    /// construct the book's structure in the form of a `Vec<BookItem>` and then calls `render()`
    /// method of the current renderer.
    ///
    /// It is the renderer who generates all the output files.

    pub fn build(&mut self) -> Result<(), Box<Error>> {
        debug!("[fn]: build");

        try!(self.init());

        // Clean output directory
        try!(utils::remove_dir_content(&self.config.get_dest()));

        try!(self.renderer.render(&self));

        Ok(())
    }


    pub fn copy_theme(&self) -> Result<(), Box<Error>> {
        debug!("[fn]: copy_theme");

        let theme_dir = self.config.get_src().join("theme");

        if !theme_dir.exists() {
            debug!("[*]: {:?} does not exist, trying to create directory", theme_dir);
            try!(fs::create_dir(&theme_dir));
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
        let root = self.config.get_root().to_owned();
        self.config.read_config(&root);
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

        // Handle absolute and relative paths
        match dest.is_absolute() {
            true => { self.config.set_dest(dest); },
            false => {
                let dest = self.config.get_root().join(dest).to_owned();
                self.config.set_dest(&dest);
            }
        }

        self
    }

    pub fn get_dest(&self) -> &Path {
        self.config.get_dest()
    }

    pub fn set_src(mut self, src: &Path) -> Self {

        // Handle absolute and relative paths
        match src.is_absolute() {
            true => { self.config.set_src(src); },
            false => {
                let src = self.config.get_root().join(src).to_owned();
                self.config.set_src(&src);
            }
        }

        self
    }

    pub fn get_src(&self) -> &Path {
        self.config.get_src()
    }

    pub fn set_title(mut self, title: &str) -> Self {
        self.config.title = title.to_owned();
        self
    }

    pub fn get_title(&self) -> &str {
        &self.config.title
    }

    pub fn set_author(mut self, author: &str) -> Self {
        self.config.author = author.to_owned();
        self
    }

    pub fn get_author(&self) -> &str {
        &self.config.author
    }

    // Construct book
    fn parse_summary(&mut self) -> Result<(), Box<Error>> {
        // When append becomes stable, use self.content.append() ...
        self.content = try!(parse::construct_bookitems(&self.config.get_src().join("SUMMARY.md")));
        Ok(())
    }

}
