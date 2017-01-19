extern crate regex;
extern crate toml;

use regex::Regex;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::collections::BTreeMap;

use utils;
use book::bookconfig::Author;

use utils::fs::create_with_str;

/// The Chapter struct holds the title of the chapter as written in the
/// SUMMARY.md file, the location of the markdown file and other metadata.
///
/// If the markdown file starts with a TOML header, it will be parsed to set the
/// chapter's properties. A TOML header should start and end with `+++` lines:
///
/// ```text
/// +++
/// title = "The Library of Babel"
/// author = "Jorge Luis Borges"
/// translator = "James E. Irby"
/// +++
///
/// # Babel
///
/// The universe (which others call the Library) is composed of an indefinite and
/// perhaps infinite number of hexagonal galleries, with vast air shafts between,
/// surrounded by very low railings. From any of the hexagons one can see,
/// interminably, the upper and lower floors.
/// ```
#[derive(Debug, Clone)]
pub struct Chapter {

    /// The title of the chapter.
    pub title: String,

    /// The Markdown content of the chapter without the optional TOML header.
    pub content: Option<String>,

    /// Path to the chapter's markdown file, relative to the book's source
    /// directory.
    ///
    /// Use `.content` to access the Markdown text when possible, instead of
    /// reading the Markdown file with `.src_path`.
    ///
    /// `book.get_src_base().join(chapter.get_src_path())` should point to the
    /// Markdown file.
    ///
    /// This way if the user had a custom folder structure in their source
    /// folder, this is re-created in the destination folder.
    ///
    /// When this is `None`, the chapter is treated as as draft. An output file
    /// is not rendered, but it appears in the TOC grayed out.
    src_path: Option<PathBuf>,

    /// Destination path to write to, relative to the book's source directory.
    ///
    /// `book.get_dest_base().join(chapter.get_dest_path())` should point to the
    /// output HTML file.
    dest_path: Option<PathBuf>,

    /// Links to the corresponding translations.
    pub translation_links: Option<Vec<TranslationLink>>,

    /// An identifier string that can allow linking translations with different paths.
    pub translation_id: Option<String>,

    /// The author of the chapter, or the book.
    pub authors: Option<Vec<Author>>,

    /// The translators of the chapter, or the book.
    pub translators: Option<Vec<Author>>,

    /// The description of the chapter.
    pub description: Option<String>,

    /// CSS class that will be added to the page-level wrap div to allow
    /// customized chapter styles.
    pub css_class: Option<String>,
}

impl Default for Chapter {
    fn default() -> Chapter {
        Chapter {
            title: "Untitled".to_string(),
            content: None,
            src_path: None,
            dest_path: None,
            translation_links: None,
            translation_id: None,
            authors: None,
            translators: None,
            description: None,
            css_class: None,
        }
    }
}

impl Chapter {

    pub fn new(title: String, src_path: PathBuf) -> Chapter {
        let mut chapter = Chapter::default();
        chapter.title = title;

        if src_path.as_os_str().len() > 0 {
            chapter.src_path = Some(src_path.clone());
            chapter.dest_path = Some(src_path.clone().with_extension("html"));
        } else {
            chapter.src_path = None;
            chapter.dest_path = None;
        }

        chapter
    }

    pub fn parse_or_create_using(&mut self, book_src_dir: &PathBuf) -> Result<&mut Self, String> {

        debug!("[fn] Chapter::parse_or_create() : {:?}", &self);

        if let Some(p) = self.get_src_path() {
            let src_path = &book_src_dir.join(&p).to_owned();
            if !src_path.exists() {
                debug!("[*] Creating: {:?}", src_path);
                match create_with_str(src_path, &format!("# {}", self.title)) {
                    Ok(_) => { return Ok(self); },
                    Err(_) => {
                        return Err(format!("Could not create: {:?}", src_path));
                    },
                }
            }

            let mut text = String::new();
            match File::open(src_path) {
                Err(e) => { return Err(format!("Read error: {:?}", e)); },
                Ok(mut f) => {
                    match f.read_to_string(&mut text) {
                        Ok(_) => {},
                        Err(e) => {
                            return Err(format!("Error: {:#?}", e));
                        },
                    }
                    self.content = Some(utils::strip_toml_header(&text));
                }
            }

            let re: Regex = Regex::new(r"(?ms)^\+\+\+\n(?P<toml>.*)\n\+\+\+\n").unwrap();

            match re.captures(&text) {
                Some(caps) => {
                    let toml = caps.name("toml").unwrap();
                    match utils::toml_str_to_btreemap(&toml) {
                        Ok(x) => {self.parse_from_btreemap(&x);},
                        Err(e) => {
                            error!("[*] Errors while parsing TOML: {:?}", e);
                            return Err(e);
                        }
                    }
                }
                None => {},
            }
        }

        Ok(self)
    }

    pub fn parse_from_btreemap(&mut self, data: &BTreeMap<String, toml::Value>) -> &mut Self {

        let extract_authors_from_slice = |x: &[toml::Value]| -> Vec<Author> {
            x.iter()
                .filter_map(|x| x.as_table())
                .map(|x| Author::from(x.to_owned()))
                .collect::<Vec<Author>>()
        };

        if let Some(a) = data.get("title") {
            self.title = a.to_string().replace("\"", "");
        }

        if let Some(a) = data.get("description") {
            self.description = Some(a.to_string().replace("\"", ""));
        }

        if let Some(a) = data.get("css_class") {
            self.css_class = Some(a.to_string());
        }

        if let Some(a) = data.get("translation_links") {
            if let Some(b) = a.as_slice() {
                let links: Vec<TranslationLink> = b.iter()
                    .filter_map(|x| x.as_table())
                    .map(|x| TranslationLink::from(x.to_owned()))
                    .collect::<Vec<TranslationLink>>();

                self.translation_links = Some(links);
            }
        }

        if let Some(a) = data.get("translation_id") {
            self.translation_id = Some(a.to_string().replace("\"", ""));
        }

        // Author name as a hash key.
        if let Some(a) = data.get("author") {
            if let Some(b) = a.as_str() {
                self.authors = Some(vec![Author::new(b)]);
            }
        }

        // Authors as an array of tables. This will override the above.
        if let Some(a) = data.get("authors") {
            if let Some(b) = a.as_slice() {
                self.authors = Some(extract_authors_from_slice(b));
            }
        }

        // Translator name as a hash key.
        if let Some(a) = data.get("translator") {
            if let Some(b) = a.as_str() {
                self.translators = Some(vec![Author::new(b)]);
            }
        }

        // Translators as an array of tables. This will override the above.
        if let Some(a) = data.get("translators") {
            if let Some(b) = a.as_slice() {
                self.translators = Some(extract_authors_from_slice(b));
            }
        }

        self
    }

    pub fn get_src_path(&self) -> Option<PathBuf> {
        self.src_path.clone()
    }

    pub fn set_src_path(&mut self, path: PathBuf) -> &mut Chapter {
        if path.as_os_str() == OsStr::new(".") {
            self.src_path = Some(PathBuf::from("".to_string()));
        } else {
            self.src_path = Some(path.to_owned());
        }
        self
    }

    pub fn get_dest_path(&self) -> Option<PathBuf> {
        self.dest_path.clone()
    }

    pub fn set_dest_path(&mut self, path: PathBuf) -> &mut Chapter {
        if path.as_os_str() == OsStr::new(".") {
            self.dest_path = Some(PathBuf::from("".to_string()));
        } else {
            self.dest_path = Some(path.to_owned());
        }
        self
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationLink {
    /// Language code, such as 'en' or 'fr'.
    pub code: String,
    /// The `<a href="">` link to the translation. `None` indicates that the
    /// language is part of the book, but there isn't a translation for this
    /// chapter.
    pub link: Option<String>,
}

impl Default for TranslationLink {
    fn default() -> TranslationLink {
        TranslationLink {
            code: "--".to_string(),
            link: None,
        }
    }
}

impl TranslationLink {
    pub fn new(code: String) -> TranslationLink {
        TranslationLink {
            code: code,
            link: None,
        }
    }

    pub fn new_with_link(code: String, link: String) -> TranslationLink {
        TranslationLink {
            code: code,
            link: Some(link),
        }
    }
}

impl From<toml::Table> for TranslationLink {
    fn from(data: toml::Table) -> TranslationLink {
        let mut link = TranslationLink::default();
        if let Some(x) = data.get("code") {
            link.code = x.to_string().replace("\"", "");
        }
        if let Some(x) = data.get("link") {
            link.link = Some(x.to_string().replace("\"", ""));
        } else {
            link.link = None;
        }
        link
    }
}
