use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use toml::{self, Value};
use toml::value::Table;
use serde::{Deserialize, Deserializer};

use errors::*;

/// The overall configuration object for MDBook.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Config {
    /// Metadata about the book.
    pub book: BookConfig,
    rest: Table,
}

impl Config {
    /// Load a `Config` from some string.
    pub fn from_str(src: &str) -> Result<Config> {
        toml::from_str(src).chain_err(|| Error::from("Invalid configuration file"))
    }

    /// Load the configuration file from disk.
    pub fn from_disk<P: AsRef<Path>>(config_file: P) -> Result<Config> {
        let mut buffer = String::new();
        File::open(config_file).chain_err(|| "Unable to open the configuration file")?
                               .read_to_string(&mut buffer)
                               .chain_err(|| "Couldn't read the file")?;

        Config::from_str(&buffer)
    }

    /// Fetch an arbitrary item from the `Config` as a `toml::Value`.
    ///
    /// You can use dotted indices to access nested items (e.g.
    /// `output.html.playpen` will fetch the "playpen" out of the html output
    /// table).
    pub fn get(&self, key: &str) -> Option<&Value> {
        let pieces: Vec<_> = key.split(".").collect();
        recursive_get(&pieces, &self.rest)
    }

    /// Fetch a value from the `Config` so you can mutate it.
    pub fn get_mut<'a>(&'a mut self, key: &str) -> Option<&'a mut Value> {
        let pieces: Vec<_> = key.split(".").collect();
        recursive_get_mut(&pieces, &mut self.rest)
    }

    /// Convenience method for getting the html renderer's configuration.
    ///
    /// # Note
    ///
    /// This is for compatibility only. It will be removed completely once the
    /// rendering and plugin system is established.
    pub fn html_config(&self) -> Option<HtmlConfig> {
        self.get_deserialized("output.html").ok()
    }

    /// Convenience function to fetch a value from the config and deserialize it
    /// into some arbitrary type.
    pub fn get_deserialized<'de, T: Deserialize<'de>, S: AsRef<str>>(&self, name: S) -> Result<T> {
        let name = name.as_ref();

        if let Some(value) = self.get(name) {
            value.clone()
                 .try_into()
                 .chain_err(|| "Couldn't deserialize the value")
        } else {
            bail!("Key not found, {:?}", name)
        }
    }

    fn from_legacy(mut table: Table) -> Config {
        let mut cfg = Config::default();

        // we use a macro here instead of a normal loop because the $out
        // variable can be different types. This way we can make type inference
        // figure out what try_into() deserializes to.
        macro_rules! get_and_insert {
            ($table:expr, $key:expr => $out:expr) => {
                if let Some(value) = $table.remove($key).and_then(|v| v.try_into().ok()) {
                    $out = value;
                }
            };
        }

        get_and_insert!(table, "title" => cfg.book.title);
        get_and_insert!(table, "authors" => cfg.book.authors);
        get_and_insert!(table, "source" => cfg.book.src);
        get_and_insert!(table, "description" => cfg.book.description);

        // This complicated chain of and_then's is so we can move
        // "output.html.destination" to "book.build_dir" and parse it into a
        // PathBuf.
        let destination: Option<PathBuf> = table.get_mut("output")
            .and_then(|output| output.as_table_mut())
            .and_then(|output| output.get_mut("html"))
            .and_then(|html| html.as_table_mut())
            .and_then(|html| html.remove("destination"))
            .and_then(|dest| dest.try_into().ok());

        if let Some(dest) = destination {
            cfg.book.build_dir = dest;
        }

        cfg.rest = table;
        cfg
    }
}

fn recursive_get<'a>(key: &[&str], table: &'a Table) -> Option<&'a Value> {
    if key.is_empty() {
        return None;
    } else if key.len() == 1 {
        return table.get(key[0]);
    }

    let first = key[0];
    let rest = &key[1..];

    if let Some(&Value::Table(ref nested)) = table.get(first) {
        recursive_get(rest, nested)
    } else {
        None
    }
}

fn recursive_get_mut<'a>(key: &[&str], table: &'a mut Table) -> Option<&'a mut Value> {
    // TODO: Figure out how to abstract over mutability to reduce copy-pasta
    if key.is_empty() {
        return None;
    } else if key.len() == 1 {
        return table.get_mut(key[0]);
    }

    let first = key[0];
    let rest = &key[1..];

    if let Some(&mut Value::Table(ref mut nested)) = table.get_mut(first) {
        recursive_get_mut(rest, nested)
    } else {
        None
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(de: D) -> ::std::result::Result<Self, D::Error> {
        let raw = Value::deserialize(de)?;

        let mut table = match raw {
            Value::Table(t) => t,
            _ => {
                use serde::de::Error;
                return Err(D::Error::custom(
                    "A config file should always be a toml table",
                ));
            }
        };

        if is_legacy_format(&table) {
            warn!("It looks like you are using the legacy book.toml format.");
            warn!("We'll parse it for now, but you should probably convert to the new format.");
            warn!("See the mdbook documentation for more details, although as a rule of thumb");
            warn!("just move all top level configuration entries like `title`, `author` and ");
            warn!("`description` under a table called `[book]` and it should all work.");
            warn!("Documentation: http://rust-lang-nursery.github.io/mdBook/format/config.html");
            return Ok(Config::from_legacy(table));
        }

        let book: BookConfig = table.remove("book")
                                    .and_then(|value| value.try_into().ok())
                                    .unwrap_or_default();
        Ok(Config {
            book: book,
            rest: table,
        })
    }
}

fn is_legacy_format(table: &Table) -> bool {
    let top_level_items = ["title", "author", "authors"];

    top_level_items.iter().any(|key| table.contains_key(&key.to_string()))
}


/// Configuration options which are specific to the book and required for
/// loading it from disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct BookConfig {
    /// The book's title.
    pub title: Option<String>,
    /// The book's authors.
    pub authors: Vec<String>,
    /// An optional description for the book.
    pub description: Option<String>,
    /// Location of the book source relative to the book's root directory.
    pub src: PathBuf,
    /// Where to put built artefacts relative to the book's root directory.
    pub build_dir: PathBuf,
    /// Does this book support more than one language?
    pub multilingual: bool,
}

impl Default for BookConfig {
    fn default() -> BookConfig {
        BookConfig {
            title: None,
            authors: Vec::new(),
            description: None,
            src: PathBuf::from("src"),
            build_dir: PathBuf::from("book"),
            multilingual: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct HtmlConfig {
    pub theme: Option<PathBuf>,
    pub curly_quotes: bool,
    pub mathjax_support: bool,
    pub google_analytics: Option<String>,
    pub additional_css: Vec<PathBuf>,
    pub additional_js: Vec<PathBuf>,
    pub playpen: Playpen,
    pub search: Search,
}

/// Configuration for tweaking how the the HTML renderer handles the playpen.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Playpen {
    pub editor: PathBuf,
    pub editable: bool,
}

/// Configuration of the search functionality of the HTML renderer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Search {
    /// Enable in browser searching. Default: true.
    pub enable: bool,
    /// Maximum number of visible results. Default: 30.
    pub limit_results: u32,
    /// The number of words used for a search result teaser. Default: 30,
    pub teaser_word_count: u32,
    /// Define the logical link between multiple search words.
    /// If true, all search words must appear in each result. Default: true.
    pub use_boolean_and: bool,
    /// Boost factor for the search result score if a search word appears in the header.
    /// Default: 2.
    pub boost_title: u8,
    /// Boost factor for the search result score if a search word appears in the hierarchy.
    /// The hierarchy contains all titles of the parent documents and all parent headings.
    /// Default: 1.
    pub boost_hierarchy: u8,
    /// Boost factor for the search result score if a search word appears in the text.
    /// Default: 1.
    pub boost_paragraph: u8,
    /// True if the searchword `micro` should match `microwave`. Default: true.
    pub expand : bool,
    /// Documents are split into smaller parts, seperated by headings. This defines, until which
    /// level of heading documents should be split. Default: 3. (`### This is a level 3 heading`)
    pub split_until_heading: u8,
}

impl Default for Search {
    fn default() -> Search {
        // Please update the documentation of `Search` when changing values!
        Search {
            enable: true,
            limit_results: 30,
            teaser_word_count: 30,
            use_boolean_and: false,
            boost_title: 2,
            boost_hierarchy: 1,
            boost_paragraph: 1,
            expand: true,
            split_until_heading: 3,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const COMPLEX_CONFIG: &'static str = r#"
        [book]
        title = "Some Book"
        authors = ["Michael-F-Bryan <michaelfbryan@gmail.com>"]
        description = "A completely useless book"
        multilingual = true
        src = "source"
        build-dir = "outputs"

        [output.html]
        theme = "./themedir"
        curly-quotes = true
        google-analytics = "123456"
        additional-css = ["./foo/bar/baz.css"]

        [output.html.playpen]
        editable = true
        editor = "ace"
        "#;

    #[test]
    fn load_a_complex_config_file() {
        let src = COMPLEX_CONFIG;

        let book_should_be = BookConfig {
            title: Some(String::from("Some Book")),
            authors: vec![String::from("Michael-F-Bryan <michaelfbryan@gmail.com>")],
            description: Some(String::from("A completely useless book")),
            multilingual: true,
            src: PathBuf::from("source"),
            build_dir: PathBuf::from("outputs"),
            ..Default::default()
        };
        let playpen_should_be = Playpen {
            editable: true,
            editor: PathBuf::from("ace"),
        };
        let html_should_be = HtmlConfig {
            curly_quotes: true,
            google_analytics: Some(String::from("123456")),
            additional_css: vec![PathBuf::from("./foo/bar/baz.css")],
            theme: Some(PathBuf::from("./themedir")),
            playpen: playpen_should_be,
            ..Default::default()
        };

        let got = Config::from_str(src).unwrap();

        assert_eq!(got.book, book_should_be);
        assert_eq!(got.html_config().unwrap(), html_should_be);
    }

    #[test]
    fn load_arbitrary_output_type() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct RandomOutput {
            foo: u32,
            bar: String,
            baz: Vec<bool>,
        }

        let src = r#"
        [output.random]
        foo = 5
        bar = "Hello World"
        baz = [true, true, false]
        "#;

        let should_be = RandomOutput {
            foo: 5,
            bar: String::from("Hello World"),
            baz: vec![true, true, false],
        };

        let cfg = Config::from_str(src).unwrap();
        let got: RandomOutput = cfg.get_deserialized("output.random").unwrap();

        assert_eq!(got, should_be);

        let baz: Vec<bool> = cfg.get_deserialized("output.random.baz").unwrap();
        let baz_should_be = vec![true, true, false];

        assert_eq!(baz, baz_should_be);
    }

    #[test]
    fn mutate_some_stuff() {
        // really this is just a sanity check to make sure the borrow checker
        // is happy...
        let src = COMPLEX_CONFIG;
        let mut config = Config::from_str(src).unwrap();
        let key = "output.html.playpen.editable";

        assert_eq!(config.get(key).unwrap(), &Value::Boolean(true));
        *config.get_mut(key).unwrap() = Value::Boolean(false);
        assert_eq!(config.get(key).unwrap(), &Value::Boolean(false));
    }

    /// The config file format has slightly changed (metadata stuff is now under
    /// the `book` table instead of being at the top level) so we're adding a
    /// **temporary** compatibility check. You should be able to still load the
    /// old format, emitting a warning.
    #[test]
    fn can_still_load_the_previous_format() {
        let src = r#"
        title = "mdBook Documentation"
        description = "Create book from markdown files. Like Gitbook but implemented in Rust"
        authors = ["Mathieu David"]
        source = "./source"

        [output.html]
        destination = "my-book" # the output files will be generated in `root/my-book` instead of `root/book`
        theme = "my-theme"
        curly-quotes = true
        google-analytics = "123456"
        additional-css = ["custom.css", "custom2.css"]
        additional-js = ["custom.js"]
        "#;

        let book_should_be = BookConfig {
            title: Some(String::from("mdBook Documentation")),
            description: Some(String::from(
                "Create book from markdown files. Like Gitbook but implemented in Rust",
            )),
            authors: vec![String::from("Mathieu David")],
            build_dir: PathBuf::from("my-book"),
            src: PathBuf::from("./source"),
            ..Default::default()
        };

        let html_should_be = HtmlConfig {
            theme: Some(PathBuf::from("my-theme")),
            curly_quotes: true,
            google_analytics: Some(String::from("123456")),
            additional_css: vec![PathBuf::from("custom.css"), PathBuf::from("custom2.css")],
            additional_js: vec![PathBuf::from("custom.js")],
            ..Default::default()
        };

        let got = Config::from_str(src).unwrap();
        assert_eq!(got.book, book_should_be);
        assert_eq!(got.html_config().unwrap(), html_should_be);
    }
}
