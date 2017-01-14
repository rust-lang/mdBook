extern crate toml;

pub mod book;
pub mod bookconfig;
pub mod toc;
pub mod chapter;

pub use self::book::Book;
use renderer::{Renderer, HtmlHandlebars};

use self::chapter::TranslationLink;
use self::toc::{TocItem, TocContent};
use utils;

use std::env;
use std::process::exit;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Read;
use std::error::Error;
use std::collections::{HashMap, BTreeMap};

#[derive(Debug, Clone)]
pub struct MDBook {

    /// Top-level directory of the book project, as an absolute path. Defaults
    /// to the current directory. `set_project_root()` converts relative paths
    /// to absolute.
    project_root: PathBuf,

    /// Path to the template for the renderer, relative to `project_root`.
    /// The `render_intent` determines its default value.
    ///
    /// A book doesn't necessarily has to have the template files. When not
    /// found in the book's folder, the embedded static assets will be used.
    ///
    /// Html Handlebars: `project_root` + `assets/_html-template`.
    template_dir: PathBuf,

    /// Input base for all books, relative to `project_root`. Defaults to `src`.
    src_base: PathBuf,

    /// Output base for all books, relative to `project_root`. Defaults to
    /// `book`.
    dest_base: PathBuf,

    /// Informs other functions which renderer has been selected, either by
    /// default or CLI argument.
    render_intent: RenderIntent,

    /// The book, or books in case of translations, accessible with a String
    /// key. The keys are expected to be the same as the two-letter codes of the
    /// language, such as 'en' or 'fr'.
    ///
    /// The String keys will be sub-folders where the translation's Markdown
    /// sources are expected.
    ///
    /// Each translation should have its own SUMMARY.md file, in its source
    /// folder with the chapter files.
    ///
    /// In the case of a single language, it is the sole item in the HashMap,
    /// and its source is not expected to be under a sub-folder, just simply in
    /// `./src`.
    ///
    /// Translations have to be declared in `book.toml` in their separate
    /// blocks. In this case `is_main_book = true` has to be set to mark the
    /// main book to avoid ambiguity.
    ///
    /// For a single language, the book's properties can be set on the main
    /// block:
    ///
    /// ```toml
    /// title = "Alice in Wonderland"
    /// author = "Lewis Carroll"
    /// ```
    ///
    /// For multiple languages, declare them in blocks:
    ///
    /// ```toml
    /// [[translations.en]]
    /// title = "Alice in Wonderland"
    /// author = "Lewis Carroll"
    /// language = { name = "English", code = "en" }
    /// is_main_book = true
    ///
    /// [[translations.fr]]
    /// title = "Alice au pays des merveilles"
    /// author = "Lewis Carroll"
    /// translator = "Henri Bué"
    /// language = { name = "Français", code = "fr" }
    ///
    /// [[translations.hu]]
    /// title = "Alice Csodaországban"
    /// author = "Lewis Carroll"
    /// translator = "Kosztolányi Dezső"
    /// language = { name = "Hungarian", code = "hu" }
    /// ```
    pub translations: HashMap<String, Book>,

    /// Space indentation in SUMMARY.md, defaults to 4 spaces.
    pub indent_spaces: i32,

    /// The `<script>` tag to insert in the render template. It is used with the
    /// 'serve' command, which is responsible for setting it.
    pub livereload_script: Option<String>,
}

impl Default for MDBook {
    fn default() -> MDBook {
        let mut proj: MDBook = MDBook {
            project_root: PathBuf::from("".to_string()),
            template_dir: PathBuf::from("".to_string()),
            src_base: PathBuf::from("src".to_string()),
            dest_base: PathBuf::from("book".to_string()),
            render_intent: RenderIntent::HtmlHandlebars,
            translations: HashMap::new(),
            indent_spaces: 4,
            livereload_script: None,
        };
        proj.set_project_root(&env::current_dir().unwrap());
        // sets default template_dir
        proj.set_render_intent(RenderIntent::HtmlHandlebars);
        proj
    }
}

#[derive(Debug, Clone)]
pub enum RenderIntent {
    HtmlHandlebars,
}

impl MDBook {

    /// Create a new `MDBook` struct with top-level project directory `project_root`
    pub fn new(project_root: &PathBuf) -> MDBook {
        MDBook::default().set_project_root(project_root).clone()
    }

    /// `init()` creates some boilerplate files and directories to get you started with your book.
    ///
    /// ```text
    /// book-example/
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

        if !self.project_root.exists() {
            fs::create_dir_all(&self.project_root).unwrap();
            info!("{:?} created", &self.project_root);
        }

        // Read book.toml if exists and populate .translations
        self.read_config();

        debug!("[*]: init done");
        Ok(())
    }

    /// Parses the `book.toml` file (if it exists) to extract the configuration parameters.
    /// The `book.toml` file should be in the root directory of the book project.
    /// The project root directory is the one specified when creating a new `MDBook`
    ///
    /// ```no_run
    /// # extern crate mdbook;
    /// # use mdbook::MDBook;
    /// # use std::path::Path;
    /// # fn main() {
    /// let mut book = MDBook::new(Path::new("project_root_dir"));
    /// # }
    /// ```
    ///
    /// In this example, `project_root_dir` will be the root directory of our book and is specified in function
    /// of the current working directory by using a relative path instead of an absolute path.
    pub fn read_config(&mut self) -> &mut Self {

        debug!("[fn]: read_config");

        // TODO refactor to a helper that returns Result?

        // TODO Maybe some error handling instead of exit(2), although it is a
        // clear indication for the user that something is wrong and we can't
        // fix it for them.

        let read_file = |path: PathBuf| -> String {
            let mut data = String::new();
            let mut f: File = match File::open(&path) {
                Ok(x) => x,
                Err(_) => {
                    error!("[*]: Failed to open {:?}", &path);
                    exit(2);
                }
            };
            if let Err(_) = f.read_to_string(&mut data) {
                error!("[*]: Failed to read {:?}", &path);
                exit(2);
            }
            data
        };

        // Read book.toml or book.json if exists to a BTreeMap

        if Path::new(self.project_root.join("book.toml").as_os_str()).exists() {

            debug!("[*]: Reading config");
            let text = read_file(self.project_root.join("book.toml"));

            match utils::toml_str_to_btreemap(&text) {
                Ok(x) => {self.parse_from_btreemap(&x);},
                Err(e) => {
                    error!("[*] Errors while parsing TOML: {:?}", e);
                    exit(2);
                }
            }

        } else if Path::new(self.project_root.join("book.json").as_os_str()).exists() {

            debug!("[*]: Reading config");
            let text = read_file(self.project_root.join("book.json"));

            match utils::json_str_to_btreemap(&text) {
                Ok(x) => {self.parse_from_btreemap(&x);},
                Err(e) => {
                    error!("[*] Errors while parsing JSON: {:?}", e);
                    exit(2);
                }
            }

        } else {
            debug!("[*]: No book.toml or book.json was found, using defaults.");
        }

        self
    }

    /// Configures MDBook properties and translations.
    ///
    /// After parsing properties for MDBook struct, it removes them from the
    /// config (template_dir, livereload, etc.). The remaining keys on the main
    /// block will be interpreted as properties of the main book.
    ///
    /// `project_root` is ignored.
    ///
    /// - dest_base
    /// - render_intent
    /// - template_dir
    /// - indent_spaces
    /// - livereload
    pub fn parse_from_btreemap(&mut self, conf: &BTreeMap<String, toml::Value>) -> &mut Self {

        let mut config = conf.clone();

        if config.contains_key("project_root") {
            config.remove("project_root");
        }

        if let Some(a) = config.get("src_base") {
            self.set_src_base(&PathBuf::from(&a.to_string()));
        }
        config.remove("src_base");

        if let Some(a) = config.get("dest_base") {
            self.set_dest_base(&PathBuf::from(&a.to_string()));
        }
        config.remove("dest_base");

        if let Some(a) = config.get("render_intent") {
            if a.to_string() == "html".to_string() {
                self.set_render_intent(RenderIntent::HtmlHandlebars);
            } else {
                // offer some real choices later on...
                self.set_render_intent(RenderIntent::HtmlHandlebars);
            }
        }
        config.remove("render_intent");

        // Parsing template_dir must be after render_intent, otherwise
        // .set_render_intent() will always override template_dir with its
        // default setting.
        if let Some(a) = config.get("template_dir") {
            self.set_template_dir(&PathBuf::from(&a.to_string()));
        }
        config.remove("template_dir");

        if let Some(a) = config.get("indent_spaces") {
            if let Some(b) = a.as_integer() {
                self.indent_spaces = b as i32;
            }
        }
        config.remove("indent_spaces");

        // If there is a 'translations' table, configugre each book from that.
        // If there isn't, take the rest of the config as one book.

        // If there is only one book, leave its source and destination folder as
        // the default `./src` and `./book`. If there are more, join their hash
        // keys to the default source and destination folder such as `/src/en`
        // and `./book/en`. This may be overridden if set specifically.

        // TODO if no is_main_book = true was set in the config, find the first
        // translation (as in the config) and mark it as the main.

        if let Some(a) = config.get("translations") {
            if let Some(b) = a.as_table() {

                let is_multilang: bool = b.iter().count() > 1;

                for (key, conf) in b.iter() {
                    let mut book = Book::new(&self.project_root);

                    if let Some(c) = conf.as_slice() {
                        if let Some(d) = c[0].as_table() {
                            if is_multilang {
                                book.config.src = book.config.src.join(key);
                                book.config.dest = book.config.dest.join(key);
                            }
                            book.config.is_multilang = is_multilang;
                            book.config.parse_from_btreemap(&d);
                            self.translations.insert(key.to_owned(), book);
                        }
                    }
                }
            }
        } else {
            let mut book = Book::new(&self.project_root);

            book.config.parse_from_btreemap(&config);
            let key = book.config.language.code.clone();
            self.translations.insert(key, book);
        }

        self
    }

    pub fn parse_books(&mut self) -> &mut Self {
        debug!("[fn]: parse_books");

        for key in self.translations.clone().keys() {
            if let Some(mut b) = self.translations.clone().get_mut(key) {

                // TODO error handling could be better here

                let first_as_index = match self.render_intent {
                    RenderIntent::HtmlHandlebars => true,
                };

                match b.parse_or_create_summary_file(first_as_index) {
                    Ok(_) => {},
                    Err(e) => {println!("{}", e);},
                }

                match b.parse_or_create_chapter_files() {
                    Ok(_) => {},
                    Err(e) => {println!("{}", e);},
                }

                self.translations.remove(key);
                self.translations.insert(key.to_owned(), b.clone());
            }
        }

        self
    }

    pub fn link_translations(&mut self) -> &mut MDBook {
        if self.translations.keys().count() == 1 {
            return self;
        }

        for (key, book) in self.translations.clone() {
            let mut newbook: Book = book.clone();

            newbook.toc = book.toc.iter()
                .map(|item| {
                    match *item {
                        TocItem::Numbered(ref i) =>
                            TocItem::Numbered(self.set_translation_links(i)),
                        TocItem::Unnumbered(ref i) =>
                            TocItem::Unnumbered(self.set_translation_links(i)),
                        TocItem::Unlisted(ref i) =>
                            TocItem::Unlisted(self.set_translation_links(i)),
                        TocItem::Spacer =>
                            TocItem::Spacer,
                    }
                }).collect::<Vec<TocItem>>();

            self.translations.remove(&key);
            self.translations.insert(key, newbook);
        }

        self
    }

    /// prepare a Vec of default links to point to the index.html of each translation
    fn translation_index_links(&mut self) -> Vec<TranslationLink> {
        let mut default_links: Vec<TranslationLink> = vec![];

        let mut keys = self.translations.keys()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        keys.sort();

        for key in keys {
            let book = self.translations.get(&key).unwrap();

            let z = self.get_dest_base();
            let a = book.config.dest.strip_prefix(&z).unwrap();
            let b = a.join("index.html");
            let c = b.to_str().unwrap();
            let link = TranslationLink::new(key, c.to_string());
            default_links.push(link);
        }

        default_links
    }

    fn set_translation_links(&mut self, content: &TocContent) -> TocContent {
        let default_links = self.translation_index_links();

        let mut newcontent: TocContent = content.clone();

        match newcontent.chapter.translation_links {
            Some(_) => {},
            None => {
                newcontent.chapter.translation_links = Some(default_links);
            }
        }

        newcontent
    }

    pub fn get_project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn set_project_root(&mut self, path: &PathBuf) -> &mut MDBook {
        if path.is_absolute() {
            self.project_root = path.to_owned();
        } else {
            self.project_root = env::current_dir().unwrap().join(path).to_owned();
        }
        self
    }

    pub fn get_template_dir(&self) -> PathBuf {
        self.project_root.join(&self.template_dir)
    }

    pub fn set_template_dir(&mut self, path: &PathBuf) -> &mut MDBook {
        if path.as_os_str() == OsStr::new(".") {
            self.template_dir = PathBuf::from("".to_string());
        } else {
            self.template_dir = path.to_owned();
        }
        self
    }

    pub fn get_src_base(&self) -> PathBuf {
        self.project_root.join(&self.src_base)
    }

    pub fn set_src_base(&mut self, path: &PathBuf) -> &mut MDBook {
        if path.as_os_str() == OsStr::new(".") {
            self.src_base = PathBuf::from("".to_string());
        } else {
            self.src_base = path.to_owned();
        }
        self
    }

    pub fn get_dest_base(&self) -> PathBuf {
        self.project_root.join(&self.dest_base)
    }

    pub fn set_dest_base(&mut self, path: &PathBuf) -> &mut MDBook {
        if path.as_os_str() == OsStr::new(".") {
            self.dest_base = PathBuf::from("".to_string());
        } else {
            self.dest_base = path.to_owned();
        }
        self
    }

    pub fn get_render_intent(&self) -> &RenderIntent {
        &self.render_intent
    }

    pub fn set_render_intent(&mut self, intent: RenderIntent) -> &mut MDBook {
        self.render_intent = intent;
        match self.render_intent {
            RenderIntent::HtmlHandlebars => {
                self.set_template_dir(&PathBuf::from("assets").join("_html-template"));
            },
        }
        self
    }

    // TODO update

    // pub fn test(&mut self) -> Result<(), Box<Error>> {
    //     // read in the chapters
    //     try!(self.parse_summary());
    //     for item in self.iter() {

    //         match *item {
    //             BookItem::Chapter(_, ref ch) => {
    //                 if ch.path != PathBuf::new() {

    //                     let path = self.get_src().join(&ch.path);

    //                     println!("[*]: Testing file: {:?}", path);

    //                     let output_result = Command::new("rustdoc")
    //                                             .arg(&path)
    //                                             .arg("--test")
    //                                             .output();
    //                     let output = try!(output_result);

    //                     if !output.status.success() {
    //                         return Err(Box::new(io::Error::new(ErrorKind::Other, format!(
    //                                         "{}\n{}",
    //                                         String::from_utf8_lossy(&output.stdout),
    //                                         String::from_utf8_lossy(&output.stderr)))) as Box<Error>);
    //                     }
    //                 }
    //             },
    //             _ => {},
    //         }
    //     }
    //     Ok(())
    // }

    // /// Returns a flat depth-first iterator over the elements of the book, it returns an [BookItem enum](bookitem.html):
    // /// `(section: String, bookitem: &BookItem)`
    // ///
    // /// ```no_run
    // /// # extern crate mdbook;
    // /// # use mdbook::MDBook;
    // /// # use mdbook::BookItem;
    // /// # use std::path::Path;
    // /// # fn main() {
    // /// # let mut book = MDBook::new(Path::new("mybook"));
    // /// for item in book.iter() {
    // ///     match item {
    // ///         &BookItem::Chapter(ref section, ref chapter) => {},
    // ///         &BookItem::Affix(ref chapter) => {},
    // ///         &BookItem::Spacer => {},
    // ///     }
    // /// }
    // ///
    // /// // would print something like this:
    // /// // 1. Chapter 1
    // /// // 1.1 Sub Chapter
    // /// // 1.2 Sub Chapter
    // /// // 2. Chapter 2
    // /// //
    // /// // etc.
    // /// # }
    // /// ```

    // pub fn iter(&self) -> BookItems {
    //     BookItems {
    //         items: &self.content[..],
    //         current_index: 0,
    //         stack: Vec::new(),
    //     }
    // }

}
