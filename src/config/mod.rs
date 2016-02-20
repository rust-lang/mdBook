//! # Configuration file
//!
//! This module handles the loading of the configuration from the `book.toml` configuration file.
//! The configuration file looks like this:
//!
//! ```toml
//! title = "mdBook"
//! description = """
//! This is a command line utility to generate books from markdown files
//! """
//!
//! [[author]]
//! name = "Mathieu David"
//! email = "mathieudavid@mathieudavid.org"
//! # other fields could be added in the future
//!
//! [source]
//! path = "src/"
//!
//! # "outputs" is a table where each entry is the identifier of a renderer
//! # containing the configuration options for that renderer
//! [outputs]
//! html = { path = "book/" }
//! # OR alternatively
//! # [outputs.html]
//! # path = "book/"
//!
//! [languages]
//! en = { name = "English", default = true }
//! fr = { name = "Français" }
//! # OR alternatively
//! # [languages.en]
//! # name = "English"
//! # default = true
//! #
//! # [languages.fr]
//! # name = "Français"
//!
//! [plugins]
//! syntax-highlighting = { enabled = true, default_language = "rust" }
//! code-line-hiding = { enabled = true, hide_pattern = "#" }
//! rust-playpen = { enabled = true }
//! # OR alternatively
//! # [plugins.syntax-highlighting]
//! # enabled = true
//! # default_language = "rust"
//! #
//! # [plugins.code-line-hiding]
//! # ...

extern crate toml;

use std::path::{Path, PathBuf};
use std::error::Error;

use utils;

pub struct Config {
    title: String,
    description: String,

    authors: Vec<Author>,

    root: PathBuf,
    source: PathBuf,

    outputs: Vec<Output>,

    language: Language,
    translations: Vec<Language>,

    plugins: Vec<Plugin>,
}

pub struct Author {
    name: String,
    email: Option<String>,
}

pub struct Output {
    identifier: String,
    destination: PathBuf,
    config: Option<toml::Table>,
}

pub struct Language {
    name: String,
    code: String,
}

pub struct Plugin {
    identifier: String,
    enabled: bool,
    config: Option<toml::Table>,
}


impl Config {
    pub fn new() -> Self {
        Config {
            title: String::new(),
            description: String::new(),

            authors: vec![],

            root: PathBuf::new(),
            source: PathBuf::new(),

            outputs: vec![],

            language: Language::default(),
            translations: vec![],

            plugins: vec![],
        }
    }

    pub fn read_config(&mut self, path: &Path) -> Result<(), Box<Error>> {
        let config_content = try!(utils::fs::file_to_string(path));
        try!(self.fill_config(&config_content));

        // When all the rest succeeded, set the root path
        self.root = path.parent()
                        .expect("How can an existing file not have a parent directory?")
                        .to_owned();

        Ok(())
    }

    fn fill_config(&mut self, toml: &str) -> Result<(), Box<Error>> {
        let mut toml_parser = toml::Parser::new(toml);

        // Handle errors in the toml file
        let config = match toml_parser.parse() {
            Some(c) => c,
            None => {
                let mut error_str = format!("could not parse input as TOML\n");
                for error in toml_parser.errors.iter() {
                    let (loline, locol) = toml_parser.to_linecol(error.lo);
                    let (hiline, hicol) = toml_parser.to_linecol(error.hi);
                    error_str.push_str(&format!("{}:{}{} {}\n",
                                                loline + 1, locol + 1,
                                                if loline != hiline || locol != hicol {
                                                    format!("-{}:{}", hiline + 1,
                                                            hicol + 1)
                                                } else {
                                                    "".to_string()
                                                },
                                                error.desc));
                }

                return Err(Box::new(::std::io::Error::new(::std::io::ErrorKind::InvalidInput, error_str)))
            },
        };


        // Retrieve toml values

        self.title = title_from_toml(&config)
                        .unwrap_or(String::from("Book"));

        self.description = description_from_toml(&config)
                                .unwrap_or(String::new());

        Ok(())
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn authors(&self) -> &[Author] {
        &self.authors
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn translations(&self) -> &[Language] {
        &self.translations
    }

    pub fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }
}


impl Author {
    /// Creates a new `Author` struct with the given name. The email field will be set to `None`
    pub fn new(name: &str) -> Self {
        Author {
            name: String::from(name),
            email: None,
        }
    }

    /// Builder pattern function, chained to `new()` it sets the email adress.
    /// Used like this:
    /// ```
    /// #extern crate mdbook;
    /// #
    /// #fn main() {
    ///     let author = mdbook::config::Author::new("John Doe").set_email("john@doe.org");
    /// #}
    ///
    pub fn set_email(mut self, email: &str) -> Self {
        self.email = Some(String::from(email));
        self
    }

    /// Returns the name of the author as `&str`
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns an `Option` with the email adress of the author
    pub fn email(&self) -> Option<&str> {
        self.email.as_ref().map(String::as_ref)
    }
}


impl Output {
    pub fn new(identifier: &str, destination: &Path) -> Self {
        Output {
            identifier: String::from(identifier),
            destination: PathBuf::from(destination),
            config: None,
        }
    }

    pub fn set_config(mut self, config: toml::Table) -> Self {
        self.config = Some(config);
        self
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn destination(&self) -> &Path {
        &self.destination
    }

    pub fn config(&self) -> Option<&toml::Table> {
        self.config.as_ref().map(|x| &*x)
    }
}


impl Language {
    pub fn new(name: &str, code: &str) -> Self {
        Language {
            name: String::from(name),
            code: String::from(code),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

impl Default for Language {
    fn default() -> Self {
        Language {
            name: String::from("English"),
            code: String::from("en"),
        }
    }
}


impl Plugin {
    pub fn new(identifier: &str, enabled: bool) -> Self {
        Plugin {
            identifier: String::from(identifier),
            enabled: enabled,
            config: None,
        }
    }

    pub fn set_config(mut self, config: toml::Table) -> Self {
        self.config = Some(config);
        self
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn config(&self) -> Option<&toml::Table> {
        self.config.as_ref().map(|x| &*x)
    }
}


// Helper functions to extract values from toml
fn title_from_toml(toml: &toml::Table) -> Option<String> {
    if let Some(value) = toml.get("title") {
        if let Some(title) = value.as_str() {
            return Some(String::from(title))
        }
    }

    None
}

fn description_from_toml(toml: &toml::Table) -> Option<String> {
    if let Some(value) = toml.get("description") {
        if let Some(description) = value.as_str() {
            return Some(String::from(description))
        }
    }

    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_config() {
        let mut config = Config::new();

        let toml = r##"title = "mdBook"
description = """
This is a command line utility to generate books from markdown files"""

[[author]]
name = "Mathieu David"
email = "mathieudavid@mathieudavid.org"
# other fields could be added

[source]
path = "src/"

# "outputs" is a table where each entry is the identifier of a renderer
# containing the configuration options for that renderer
[outputs]
html = { path = "book/" }
pdf = { path = "pdf/mdBook.pdf" }
# OR alternatively
# [outputs.html]
# path = "book/"
#
# [outputs.pdf]
# path = "pdf/mdBook.pdf"

[languages]
en = { name = "English", default = true }
fr = { name = "Français" }
# OR alternatively
# [languages.en]
# name = "English"
# default = true
#
# [languages.fr]
# name = "Français"

[plugins]
syntax-highlighting = { enabled = true, default_language = "rust" }
code-line-hiding = { enabled = true, hide_pattern = "#" }
rust-playpen = { enabled = true }
# OR alternatively
# [plugins.syntax-highlighting]
# enabled = true
# default_language = "rust"
#
# [plugins.code-line-hiding]
# ...
"##;

        config.fill_config(toml);

        assert_eq!(config.title(), "mdBook");
        assert_eq!(config.description(), "This is a command line utility to generate books from markdown files");
    }


    #[test]
    fn fill_config_empty() {
        let mut config = Config::new();

        let toml = r#""#;

        config.fill_config(toml);

        assert_eq!(config.title(), "Book");
        assert_eq!(config.description(), "");
    }
}
