use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use toml::{self, Value};
use serde::Deserialize;

use errors::*;


/// The overall configuration object for MDBook.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Metadata about the book.
    pub book: BookConfig,
    /// Arbitrary information which renderers can use during the rendering
    /// stage.
    pub output: BTreeMap<String, Value>,
    /// Information for use by preprocessors.
    pub preprocess: BTreeMap<String, Value>,
    /// Information for use by postprocessors.
    pub postprocess: BTreeMap<String, Value>,
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

    /// Convenience method for getting the html renderer's configuration.
    ///
    /// # Note
    ///
    /// This is for compatibility only. It will be removed completely once the
    /// rendering and plugin system is established.
    pub fn html_config(&self) -> Option<HtmlConfig> {
        self.try_get_output("html").ok()
    }

    /// Try to get an output and deserialize it as a `T`.
    pub fn try_get_output<'de, T: Deserialize<'de>, S: AsRef<str>>(&self, name: S) -> Result<T> {
        get_deserialized(name, &self.output)
    }

    /// Try to get the configuration for a preprocessor, deserializing it as a
    /// `T`.
    pub fn try_get_preprocessor<'de, T: Deserialize<'de>, S: AsRef<str>>(&self,
                                                                         name: S)
                                                                         -> Result<T> {
        get_deserialized(name, &self.preprocess)
    }

    /// Try to get the configuration for a postprocessor, deserializing it as a
    /// `T`.
    pub fn try_get_postprocessor<'de, T: Deserialize<'de>, S: AsRef<str>>(&self,
                                                                          name: S)
                                                                          -> Result<T> {
        get_deserialized(name, &self.postprocess)
    }
}

/// Convenience function to load a value from some table then deserialize it.
fn get_deserialized<'de, T: Deserialize<'de>, S: AsRef<str>>(name: S,
                                                             table: &BTreeMap<String, Value>)
                                                             -> Result<T> {
    let name = name.as_ref();

    match table.get(name) {
        Some(output) => {
            output.clone()
                  .try_into()
                  .chain_err(|| "Couldn't deserialize the value")
        }
        None => bail!("Key Not Found, {}", name),
    }
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
    /// Location of the book source, relative to the book's root directory.
    pub src: PathBuf,
    /// Where to put built artefacts, relative to the book's root directory.
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
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Playpen {
    pub editor: PathBuf,
    pub editable: bool,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_a_complex_config_file() {
        let src = r#"
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
        let got: RandomOutput = cfg.try_get_output("random").unwrap();

        assert_eq!(got, should_be);
    }
}
