extern crate regex;
extern crate toml;

use regex::Regex;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::error::Error;
use std::io::{self, Read};
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
/// ```
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

    /// Path to the chapter's markdown file, relative to the book's source
    /// directory.
    ///
    /// `book.src.join(chapter.path)` points to the Markdown file, and
    /// `book.dest.join(chapter.path).with_extension("html")` points to the
    /// output html file. This way if the user had a custom folder structure in
    /// their source folder, this is re-created in the destination folder.
    pub path: PathBuf,

    /// Optional destination path to write to. Used when changing the first
    /// chapter's path to index.html.
    pub dest_path: Option<PathBuf>,

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
            path: PathBuf::from("src".to_string()).join("untitled.md"),
            dest_path: None,
            authors: None,
            translators: None,
            description: None,
            css_class: None,
        }
    }
}

impl Chapter {

    pub fn new(title: String, path: PathBuf) -> Chapter {
        let mut chapter = Chapter::default();
        chapter.title = title;
        chapter.path = path;
        chapter
    }

    pub fn parse_or_create_using(&mut self, book_src_dir: &PathBuf) -> Result<&mut Self, String> {

        debug!("[fn] Chapter::parse_or_create() : {:?}", &self);

        let src_path = &book_src_dir.join(&self.path).to_owned();
        if !src_path.exists() {
            debug!("[*] Creating: {:?}", src_path);
            match create_with_str(src_path, &format!("# {}", self.title)) {
                Ok(_) => { return Ok(self); },
                Err(e) => {
                    return Err(format!("Could not create: {:?}", src_path));
                },
            }
        }

        let mut text = String::new();
        match File::open(src_path) {
            Err(e) => { return Err(format!("Read error: {:?}", e)); },
            Ok(mut f) => {
                f.read_to_string(&mut text);
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

    /// Reads in the chapter's content from the markdown file. Chapter doesn't
    /// know the book's src folder, hence the `book_src_dir` argument.
    pub fn read_content_using(&self, book_src_dir: &PathBuf) -> Result<String, Box<Error>> {

        let src_path = book_src_dir.join(&self.path);

        if !src_path.exists() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Doesn't exist: {:?}", src_path))
            ));
        }

        debug!("[*]: Opening file: {:?}", src_path);

        let mut f = try!(File::open(&src_path));
        let mut content: String = String::new();

        debug!("[*]: Reading file");
        try!(f.read_to_string(&mut content));

        // Render markdown using the pulldown-cmark crate
        content = utils::strip_toml_header(&content);
        content = utils::render_markdown(&content);

        Ok(content)
    }

}
