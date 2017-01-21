extern crate regex;
extern crate toml;

use regex::Regex;

pub mod book;
pub mod bookconfig;
pub mod toc;
pub mod chapter;

pub use self::book::Book;
use self::chapter::TranslationLink;
use self::toc::{TocItem, TocContent};
use utils;

use std::env;
use std::process::exit;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
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
    /// and its Markdown files are not expected to be under a sub-folder, just
    /// simply in `./src`.
    ///
    /// Translations have to be declared in `book.toml` in their separate
    /// blocks. The first in the TOML config will be recognized as the main
    /// translation, `is_main_book` will be set to true on it.
    ///
    /// If the first in the TOML config is not the main translation, the user
    /// has to set `is_main_book = true` to mark the main book to avoid
    /// ambiguity.
    ///
    /// For a single language, the book's properties can be set on the main
    /// block:
    ///
    /// ```toml
    /// title = "Alice in Wonderland"
    /// author = "Lewis Carroll"
    /// ```
    ///
    /// For multiple languages, declare them in blocks. The translation key will
    /// be the language code. Optionally, the language name can be set as well.
    ///
    /// ```toml
    /// [[translations.en]]
    /// title = "Alice in Wonderland"
    /// author = "Lewis Carroll"
    ///
    /// [[translations.fr]]
    /// title = "Alice au pays des merveilles"
    /// author = "Lewis Carroll"
    /// translator = "Henri Bué"
    /// language_name = "Français"
    ///
    /// [[translations.hu]]
    /// title = "Alice Csodaországban"
    /// author = "Lewis Carroll"
    /// translator = "Kosztolányi Dezső"
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

    /// Parses the `book.toml` file (if it exists) to extract the configuration parameters.
    /// The `book.toml` file should be in the root directory of the book project.
    /// The project root directory is the one specified when creating a new `MDBook`
    ///
    /// ```ignore
    /// # extern crate mdbook;
    /// # use mdbook::MDBook;
    /// # use std::path::PathBuf;
    /// # fn main() {
    /// let mut book = MDBook::new(&PathBuf::from("project_root_dir"));
    /// # }
    /// ```
    ///
    /// In this example, `project_root_dir` will be the root directory of our book and is specified in function
    /// of the current working directory by using a relative path instead of an absolute path.
    pub fn read_config(&mut self) -> &mut Self {
        debug!("[fn]: read_config");

        // exit(2) is a clear indication for the user that something is wrong
        // and we can't fix it for them.

        // Read book.toml or book.json if exists to a BTreeMap

        if self.project_root.join("book.toml").exists() {

            debug!("[*]: Reading config");
            let text = match utils::fs::file_to_string(&self.project_root.join("book.toml")) {
                Ok(x) => x,
                Err(e) => {
                    error!("[*] Read error: {:#?}", e);
                    exit(2);
                }
            };

            match utils::toml_str_to_btreemap(&text) {
                Ok(x) => {self.parse_from_btreemap(&x);},
                Err(e) => {
                    error!("[*] Errors while parsing TOML: {:?}", e);
                    exit(2);
                }
            }

        } else if self.project_root.join("book.json").exists() {

            debug!("[*]: Reading config");
            let text = match utils::fs::file_to_string(&self.project_root.join("book.json")) {
                Ok(x) => x,
                Err(e) => {
                    error!("[*] Read error: {:#?}", e);
                    exit(2);
                }
            };

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

        if let Some(a) = config.get("translations") {
            if let Some(b) = a.as_table() {

                let is_multilang: bool = b.iter().count() > 1;

                let mut has_main_book_already = false;

                for (key, conf) in b.iter() {
                    let mut book = Book::new(&self.project_root);

                    if let Some(c) = conf.as_slice() {
                        if let Some(d) = c[0].as_table() {
                            if is_multilang {
                                book.config.src = book.config.src.join(key);
                                book.config.dest = book.config.dest.join(key);
                            }
                            book.config.is_multilang = is_multilang;
                            book.config.parse_from_btreemap(key.to_owned(), &d);
                            // the language code and translation key must agree
                            // even after parsing the user's settings
                            book.config.language.code = key.to_owned();
                            if book.config.is_main_book {
                                has_main_book_already = true;
                            }
                            self.translations.insert(key.to_owned(), book);
                        }
                    }
                }

                // If there hasn't been a 'is_main_book = true' set in the
                // config, we have to find the first translation as given in the
                // config and assume it to be the main book.
                //
                // Since the config is a BTreeMap, in which entries are ordered
                // by the keys, the order in which they appear in the book.toml
                // file has to be deduced by matching the file contents with a
                // Regex.

                if !has_main_book_already {
                    if Path::new(self.project_root.join("book.toml").as_os_str()).exists() {

                        let text = match utils::fs::file_to_string(&self.project_root.join("book.toml")) {
                            Ok(x) => x,
                            Err(e) => {
                                error!("[*] Read error: {:#?}", e);
                                exit(2);
                            }
                        };

                        let re: Regex = Regex::new(r"\[\[translations\.(?P<key>[^]]+)\]\]").unwrap();

                        match re.captures(&text) {
                            Some(caps) => {
                                if let Some(key) = caps.name("key") {
                                    if let Some(mut a) = self.translations.get_mut(key) {
                                        a.config.is_main_book = true;
                                    }
                                }
                            },
                            None => {},
                        }

                    } else if Path::new(self.project_root.join("book.json").as_os_str()).exists() {

                        // Not going to bother with Regex-parsing JSON. We're
                        // only supporting it for <= v0.0.15 books where the format
                        // was simple and the translations key hasn't been introduced.

                        error!("When using the JSON file format for configuration, mark the main trainslation by setting the \"is_main_book\": \"true\" property. Or use the TOML format and the first translation will be recognized as the main language.");

                        exit(2);
                    }
                }

            }
        } else {
            let mut book = Book::new(&self.project_root);

            // take "en" as default code, will override if user sets it
            book.config.parse_from_btreemap("en".to_owned(), &config);
            let key = book.config.language.code.clone();
            self.translations.insert(key, book);
        }


        self
    }

    pub fn parse_books(&mut self) -> &mut Self {
        debug!("[fn]: parse_books");

        for key in self.translations.clone().keys() {
            if let Some(mut b) = self.translations.clone().get_mut(key) {

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
                            TocItem::Numbered(self.set_translation_links(i, key.to_owned())),
                        TocItem::Unnumbered(ref i) =>
                            TocItem::Unnumbered(self.set_translation_links(i, key.to_owned())),
                        TocItem::Unlisted(ref i) =>
                            TocItem::Unlisted(self.set_translation_links(i, key.to_owned())),
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
    pub fn translation_index_links(&self) -> Option<Vec<TranslationLink>> {
        debug!("[fn] translation_index_links()");

        let mut default_links: Vec<TranslationLink> = vec![];

        let mut keys = self.translations.keys()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        keys.sort();

        if keys.len() < 2 {
            // There is only one language. No need to display translation links.
            return None;
        }

        for key in keys {
            let book = self.translations.get(&key).unwrap();

            let z = self.get_dest_base();
            let a = book.config.dest.strip_prefix(&z).unwrap();
            let b = a.join("index.html");
            let c = b.to_str().unwrap();
            let link = TranslationLink::new_with_link(key, c.to_string());
            default_links.push(link);
        }

        Some(default_links)
    }

    fn set_translation_links(&mut self, content: &TocContent, key: String) -> TocContent {
        let mut final_links: BTreeMap<String, TranslationLink> = BTreeMap::new();
        let mut newcontent: TocContent = content.clone();

        // Start by adding the code of each language but no links. These will
        // render as gray <span> tags.
        for key in self.translations.keys() {
            final_links.insert(key.clone(), TranslationLink::new(key.clone()));
        }

        // Take the links parsed from the chapter's TOML header

        match newcontent.chapter.translation_links {
            Some(links) => {
                for i in links.iter() {
                    final_links.insert(i.clone().code, i.clone());
                }
            },
            None => {},
        }

        // Find a translation for the links that are still None

        let orig_book: &Book = self.translations.get(&key).unwrap();
        let orig_toc_id = toc::toc_node_count_id(&orig_book.toc);

        for (key, trl) in final_links.clone().iter() {
            match trl.link {
                Some(_) => { continue; },
                None => {},
            }

            let b: &Book = self.translations.get(key).unwrap();
            let flat_toc = toc::flat_toc(&b.toc);

            let by_section: bool = toc::toc_node_count_id(&b.toc) == orig_toc_id;

            for item in flat_toc.iter() {
                match *item {
                    TocItem::Numbered(ref i) |
                    TocItem::Unnumbered(ref i) |
                    TocItem::Unlisted(ref i) => {

                        // Note that this will also add a link to itself, which is good.

                        if content.is_it_a_translation_of(i, true, true, by_section) {
                            if let Some(mut a) = i.chapter.get_dest_path() {
                                // Join the path to the language code, i.e. en/tears.html
                                a = PathBuf::from(key.to_string()).join(a);
                                let path = String::from(a.to_str().unwrap());

                                final_links.insert(
                                    key.to_owned(),
                                    TranslationLink::new_with_link(key.to_owned(), path)
                                );
                            } else {
                                debug!("It's a translation but dest_path is not set: {:#?}", i);
                            }
                            break;
                        }
                    },
                    TocItem::Spacer => {},
                }
            }
        }

        let a: Vec<TranslationLink> = final_links.values().map(|x| x.clone()).collect();
        newcontent.chapter.translation_links = Some(a);

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

        let a = self.translations.clone();
        let keys = a.keys();
        let is_multilang: bool = keys.clone().count() > 1;

        for key in keys {
            if let Some(mut book) = self.translations.get_mut(key) {
                if is_multilang {
                    book.config.src = self.src_base.join(key);
                    book.config.is_multilang = is_multilang;
                } else {
                    book.config.src = self.src_base.to_owned();
                }
            }
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

        let a = self.translations.clone();
        let keys = a.keys();
        let is_multilang: bool = keys.clone().count() > 1;

        for key in keys {
            if let Some(mut book) = self.translations.get_mut(key) {
                if is_multilang {
                    book.config.dest = self.dest_base.join(key);
                    book.config.is_multilang = is_multilang;
                } else {
                    book.config.dest = self.dest_base.to_owned();
                }
            }
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

}

