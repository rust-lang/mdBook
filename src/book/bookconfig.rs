extern crate toml;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use utils;

#[derive(Debug, Clone)]
pub struct BookConfig {

    // Paths

    pub dest: PathBuf,
    pub src: PathBuf,

    // Metadata

    /// The title of the book.
    pub title: String,
    /// The subtitle, when the book's full title is in the form of "The Immense
    /// Journey: An Imaginative Naturalist Explores the Mysteries of Man and
    /// Nature"
    pub subtitle: Option<String>,
    /// A brief description or summary of the book.
    pub description: Option<String>,
    pub language: Language,
    pub authors: Vec<Author>,
    pub translators: Option<Vec<Author>>,
    /// Publisher's info
    pub publisher: Option<Publisher>,
    /// Chapter numbering scheme
    pub number_format: NumberFormat,
    /// Section names for nested Vec<Chapter> structures, defaults to `[ "Chapter", "Section", "Subsection" ]`
    pub section_names: Vec<String>,
    /// Whether this is the main book, in the case of translations.
    pub is_main_book: bool,
    pub is_multilang: bool,
}

impl Default for BookConfig {
    fn default() -> BookConfig {
        BookConfig {
            dest: PathBuf::from("book".to_string()),
            src: PathBuf::from("src".to_string()),

            title: "Untitled".to_string(),
            subtitle: None,
            description: None,
            language: Language::default(),
            authors: vec![Author::new("The Author").file_as("Author, The")],
            translators: None,
            publisher: None,
            number_format: NumberFormat::Arabic,
            section_names: vec!["Chapter".to_string(),
                                "Section".to_string(),
                                "Subsection".to_string()],
            is_main_book: false,
            is_multilang: false,
        }
    }
}

impl BookConfig {

    pub fn new(project_root: &PathBuf) -> BookConfig {
        let mut conf = BookConfig::default();

        // join paths to project_root
        // Prefer "" to "." and "src" to "./src", avoid "././src"

        let mut pr = project_root.clone();
        if pr.as_os_str() == OsStr::new(".") {
            pr = PathBuf::from("".to_string());
        }

        conf.dest = pr.join(&conf.dest);
        conf.src = pr.join(&conf.src);

        conf
    }

    /// Parses keys from a BTreeMap one by one. Not trying to directly
    /// de-serialize to `BookConfig` so that we can provide some convenient
    /// shorthands for the user.
    ///
    /// `book.toml` is a user interface, not an app data store, we never have to
    /// write data back to it.
    ///
    /// Parses author when given as an array, or when given as a hash key to
    /// make declaring a single author easy.
    ///
    /// Both of these express a single author:
    ///
    /// ```toml
    /// [[authors]]
    /// name = "Marcus Aurelius Antoninus"
    /// ```
    ///
    /// Or:
    ///
    /// ```toml
    /// name = "Marcus Aurelius Antoninus"
    /// ```
    ///
    pub fn parse_from_btreemap(&mut self,
                               default_language_code: String,
                               config: &BTreeMap<String, toml::Value>) -> &mut Self {

        // Paths

        // Destination folder
        if let Some(a) = config.get("dest") {
            let dest = PathBuf::from(&a.to_string().replace("\"", ""));
            self.set_dest(&dest);
        }

        // Source folder
        if let Some(a) = config.get("src") {
            let src = PathBuf::from(&a.to_string().replace("\"", ""));
            self.set_src(&src);
        }

        // Metadata

        let extract_authors_from_slice = |x: &[toml::Value]| -> Vec<Author> {
            x.iter()
                .filter_map(|x| x.as_table())
                .map(|x| Author::from(x.to_owned()))
                .collect::<Vec<Author>>()
        };

        self.language = Language::new(&default_language_code);

        if let Some(a) = config.get("title") {
            self.title = a.to_string().replace("\"", "");
        }

        if let Some(a) = config.get("subtitle") {
            self.subtitle = Some(a.to_string().replace("\"", ""));
        }

        if let Some(a) = config.get("description") {
            self.description = Some(a.to_string().replace("\"", ""));
        }

        if let Some(a) = config.get("language") {
            if let Some(b) = a.as_table() {
                self.language = Language::from(b.to_owned());
            }
        }

        if let Some(a) = config.get("language_code") {
            self.language.code = a.to_string().replace("\"", "");
        }

        if let Some(a) = config.get("language_name") {
            self.language.name = Some(a.to_string().replace("\"", ""));
        }

        // Author name as a hash key.
        if let Some(a) = config.get("author") {
            if let Some(b) = a.as_str() {
                self.authors = vec![Author::new(b)];
            }
        }

        // Authors as an array of tables. This will override the above.
        if let Some(a) = config.get("authors") {
            if let Some(b) = a.as_slice() {
                self.authors = extract_authors_from_slice(b);
            }
        }

        // Translator name as a hash key.
        if let Some(a) = config.get("translator") {
            if let Some(b) = a.as_str() {
                self.translators = Some(vec![Author::new(b)]);
            }
        }

        // Translators as an array of tables. This will override the above.
        if let Some(a) = config.get("translators") {
            if let Some(b) = a.as_slice() {
                self.translators = Some(extract_authors_from_slice(b));
            }
        }

        if let Some(a) = config.get("publisher") {
            if let Some(b) = a.as_table() {
                self.publisher = Some(Publisher::from(b.to_owned()));
            }
        }

        if let Some(a) = config.get("number_format") {
            if let Some(b) = a.as_str() {
                self.number_format = match b.to_lowercase().as_ref() {
                    "arabic" => NumberFormat::Arabic,
                    "roman" => NumberFormat::Roman,
                    "word" => NumberFormat::Word,
                    _ => NumberFormat::Arabic,
                };
            }
        }

        if let Some(a) = config.get("section_names") {
            if let Some(b) = a.as_slice() {
                self.section_names =
                    b.iter()
                    .filter_map(|x| x.as_str())
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
            }
        }

        if let Some(a) = config.get("is_main_book") {
            if let Some(b) = a.as_bool() {
                self.is_main_book = b;
            }
        }

        self
    }

    pub fn get_dest(&self) -> &Path {
        &self.dest
    }

    pub fn set_dest(&mut self, dest: &Path) -> &mut BookConfig {
        self.dest = dest.to_owned();
        self
    }

    pub fn get_src(&self) -> &Path {
        &self.src
    }

    pub fn set_src(&mut self, src: &Path) -> &mut BookConfig {
        self.src = src.to_owned();
        self
    }

}

#[derive(Debug, Clone)]
pub struct Author {
    /// Author's name, such as "Howard Philip Lovecraft"
    name: String,
    /// Author's name in the form of "Lovecraft, Howard Philip", an ebook metadata field used for sorting
    file_as: String,
    email: Option<String>,
}

impl Author {

    pub fn new(name: &str) -> Self {
        Author {
            name: name.to_owned(),
            file_as: utils::last_name_first(name),
            email: None,
        }
    }

    pub fn file_as(mut self, file_as: &str) -> Self {
        self.file_as = file_as.to_owned();
        self
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = Some(email.to_owned());
        self
    }
}

impl From<toml::Table> for Author {
    fn from(data: toml::Table) -> Author {
        let mut author = Author::new("The Author");
        if let Some(x) = data.get("name") {
            author.name = x.to_string().replace("\"", "");
        }
        if let Some(x) = data.get("file_as") {
            author.file_as = x.to_string().replace("\"", "");
        } else {
            author.file_as = utils::last_name_first(&author.name);
        }
        if let Some(x) = data.get("email") {
            author.email = Some(x.to_string().replace("\"", ""));
        }
        author
    }
}

#[derive(Debug, Clone)]
pub struct Language {
    pub code: String,
    pub name: Option<String>,
}

impl Default for Language {
    fn default() -> Self {
        Language {
            code: String::from("en"),
            name: Some(String::from("English")),
        }
    }
}

impl Language {
    pub fn new(code: &str) -> Language {
        Language{
            code: code.to_string(),
            name: None,
        }
    }

    pub fn new_with_name(code: &str, name: &str) -> Language {
        Language{
            code: code.to_string(),
            name: Some(name.to_string()),
        }
    }
}

impl From<toml::Table> for Language {
    fn from(data: toml::Table) -> Language {
        let mut language = Language::default();
        if let Some(x) = data.get("code") {
            language.code = x.to_string().replace("\"", "");
        }
        if let Some(x) = data.get("name") {
            language.name = Some(x.to_string().replace("\"", ""));
        } else {
            language.name = None;
        }
        language
    }
}

#[derive(Debug, Clone)]
pub struct Publisher {
    /// name of the publisher organization
    name: String,
    /// link to the sublisher's site
    url: Option<String>,
    /// path to publisher's logo image
    logo_src: Option<PathBuf>,
}

impl Default for Publisher {
    fn default() -> Publisher {
        Publisher {
            name: "The Publisher".to_string(),
            url: None,
            logo_src: None,
        }
    }
}

impl Publisher {
    pub fn new(name: &str) -> Publisher {
        Publisher {
            name: name.to_string(),
            url: None,
            logo_src: None,
        }
    }
}

impl From<toml::Table> for Publisher {
    fn from(data: toml::Table) -> Publisher {
        let mut publisher = Publisher::default();
        if let Some(x) = data.get("name") {
            publisher.name = x.to_string().replace("\"", "");
        }
        if let Some(x) = data.get("url") {
            publisher.url = Some(x.to_string());
        }
        if let Some(x) = data.get("logo_src") {
            publisher.logo_src = Some(PathBuf::from(x.to_string()));
        }
        publisher
    }
}

/// NumberFormat when rendering chapter titles.
#[derive(Debug, Clone)]
pub enum NumberFormat {
    /// 19
    Arabic,
    /// XIX
    Roman,
    /// Nineteen
    Word,
}

