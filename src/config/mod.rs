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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Author {
    name: String,
    email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Output {
    identifier: String,
    destination: Option<PathBuf>,
    config: Option<toml::Table>,
}

#[derive(Debug, Clone)]
pub struct Language {
    name: String,
    code: String,
}

#[derive(Debug, Clone)]
pub struct Plugin {
    identifier: String,
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

        self.source = source_from_toml(&config)
                        .and_then(|s| source_path_from_toml(&s))
                        .unwrap_or(PathBuf::from("src/"));

        self.authors = authors_from_toml(&config);

        self.outputs = outputs_from_toml(&config);

        self.language = default_language_from_toml(&config)
                            .unwrap_or(Language::new("English", "en"));

        self.translations = translations_from_toml(&config);

        self.plugins = plugins_from_toml(&config);

        Ok(())
    }

    // Title
    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = String::from(title);
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }


    // Description
    pub fn set_description(&mut self, description: &str) -> &mut Self {
        self.description = String::from(description);
        self
    }

    pub fn description(&self) -> &str {
        &self.description
    }


    // Authors
    pub fn add_author(&mut self, author: Author) -> &mut Self {
        self.authors.push(author);
        self
    }

    pub fn authors(&self) -> &[Author] {
        &self.authors
    }

    // Root
    pub fn set_root(&mut self, root: &Path) -> &mut Self {
        self.root = PathBuf::from(root);
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    // Source
    pub fn set_source(&mut self, source: &Path) -> &mut Self {
        if source.is_relative() {
            self.source = self.root.join(self.source.clone());
        } else {
            self.source = PathBuf::from(source);
        }

        self
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
    ///     let author = mdbook::config::Author::new("John Doe").set_email(Some("john@doe.org"));
    /// #}
    ///
    pub fn set_email(mut self, email: Option<&str>) -> Self {
        self.email = email.map(|s| String::from(s));
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
    pub fn new(identifier: &str) -> Self {
        Output {
            identifier: String::from(identifier),
            destination: None,
            config: None,
        }
    }

    pub fn set_output_destination(mut self, path: Option<&Path>) -> Self {
        self.destination = path.map(|p| PathBuf::from(p));
        self
    }

    pub fn set_config(mut self, config: toml::Table) -> Self {
        self.config = Some(config);
        self
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn destination(&self) -> Option<&PathBuf> {
        self.destination.as_ref()
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
    pub fn new(identifier: &str) -> Self {
        Plugin {
            identifier: String::from(identifier),
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

    pub fn config(&self) -> Option<&toml::Table> {
        self.config.as_ref().map(|x| &*x)
    }
}


// Helper functions to extract values from toml
fn title_from_toml(toml: &toml::Table) -> Option<String> {
    toml.get("title")
        .and_then(|v| v.as_str())
        .map(|v| v.to_owned())
}

fn description_from_toml(toml: &toml::Table) -> Option<String> {
    toml.get("description")
        .and_then(|v| v.as_str())
        .map(|v| v.to_owned())
}

fn source_from_toml(toml: &toml::Table) -> Option<toml::Table> {
    toml.get("source")
        .and_then(|v| v.as_table())
        .map(|v| v.to_owned())
}

fn source_path_from_toml(source: &toml::Table) -> Option<PathBuf> {
    source.get("path")
          .and_then(|v| v.as_str())
          .map(|v| PathBuf::from(v))
}

fn authors_from_toml(toml: &toml::Table) -> Vec<Author> {
    let array = toml.get("author")
                    .and_then(|v| v.as_slice())
                    .unwrap_or(&[]);

    let mut authors: Vec<Author> = vec![];

    for author in array {
        let author = if let Some(t) = author.as_table() { t } else { continue };

        let name = author.get("name")
                         .and_then(|v| v.as_str())
                         .unwrap_or("Anonymous");

        let email = author.get("email")
                          .and_then(|v| v.as_str());

        authors.push(Author::new(name).set_email(email));
    }

    authors
}

fn outputs_from_toml(toml: &toml::Table) -> Vec<Output> {
    let table = toml.get("outputs")
                    .and_then(|v| v.as_table())
                    .map(|v| v.to_owned());

    if let None = table { return Vec::new() }

    let mut outputs = Vec::new();

    for (key, config) in table.unwrap() {
        let config = if let Some(c) = config.as_table() { c } else { continue };

        // The renderer can be specified explicitely else the key is used to match the renderer
        let renderer = config.get("renderer")
                             .and_then(|v| v.as_str())
                             .unwrap_or(&key);

        let path = config.get("path")
                         .and_then(|v| v.as_str())
                         .map(|v| Path::new(v));

        let mut c = config.clone();
        c.remove("path");
        c.remove("renderer");

        outputs.push(Output::new(renderer).set_output_destination(path).set_config(c));
    }

    outputs
}

fn default_language_from_toml(toml: &toml::Table) -> Option<Language> {
    let table = toml.get("languages")
                    .and_then(|v| v.as_table())
                    .map(|v| v.to_owned());

    if let None = table { return None }

    for (language_code, language) in table.unwrap() {
        let language = if let Some(l) = language.as_table() { l } else { continue };

        if let Some(true) = language.get("default").and_then(|d| d.as_bool()) {
            let name = language.get("name")
                               .and_then(|v| v.as_str());

            if let None = name { continue }

            return Some(Language::new(name.unwrap(), &language_code));
        }
    }

    None
}

fn translations_from_toml(toml: &toml::Table) -> Vec<Language> {
    let table = toml.get("languages")
                    .and_then(|v| v.as_table())
                    .map(|v| v.to_owned());

    if let None = table { return Vec::new() }

    let mut translations = Vec::new();

    for (language_code, language) in table.unwrap() {
        let language = if let Some(l) = language.as_table() { l } else { continue };

        // Skip default language
        if let Some(true) = language.get("default").and_then(|d| d.as_bool()) { continue }

        let name = language.get("name")
                           .and_then(|v| v.as_str());

        if let None = name { continue }

        translations.push(Language::new(name.unwrap(), &language_code));

    }

    translations
}

fn plugins_from_toml(toml: &toml::Table) -> Vec<Plugin> {
    let table = toml.get("plugins")
                    .and_then(|v| v.as_table())
                    .map(|v| v.to_owned());

    if let None = table { return Vec::new() }

    let mut plugins = Vec::new();

    for (id, plugin) in table.unwrap() {
        let plugin = if let Some(l) = plugin.as_table() { l } else { continue };

        // Skip if plugin is disabled
        if let Some(false) = plugin.get("enabled").and_then(|d| d.as_bool()) { continue }

        let mut config = plugin.clone();
        config.remove("enabled");

        let mut p = Plugin::new(&id);

        if !config.is_empty() {
            p = p.set_config(config);
        }

        plugins.push(p);

    }

    plugins
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

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
path = "custom_source/"

# "outputs" is a table where each entry is the identifier of a renderer
# containing the configuration options for that renderer
[outputs]
html = { path = "book/" }
html2 = { renderer = "html", path = "book2/" }
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

        config.fill_config(toml).expect("Error while parsing the config");

        assert_eq!(config.title(), "mdBook");
        assert_eq!(config.description(), "This is a command line utility to generate books from markdown files");
        assert_eq!(config.source(), PathBuf::from("custom_source/"));
        assert_eq!(config.authors()[0].name, "Mathieu David".to_owned());
        assert_eq!(config.authors()[0].email, Some("mathieudavid@mathieudavid.org".to_owned()));
        assert_eq!(config.outputs()[0].identifier, "html");
        assert_eq!(config.outputs()[0].destination, Some(PathBuf::from("book/")));
        assert_eq!(config.outputs()[1].identifier, "html");
        assert_eq!(config.outputs()[1].destination, Some(PathBuf::from("book2/")));
        assert_eq!(config.outputs()[2].identifier, "pdf");
        assert_eq!(config.outputs()[2].destination, Some(PathBuf::from("pdf/mdBook.pdf")));
        assert_eq!(config.language.name, "English");
        assert_eq!(config.translations()[0].name, "Français");
        assert_eq!(config.plugins()[2].identifier, "syntax-highlighting");
    }


    #[test]
    fn fill_config_empty() {
        let mut config = Config::new();

        let toml = r#""#;

        config.fill_config(toml).expect("Error while parsing the config");;

        assert_eq!(config.title(), "Book");
        assert_eq!(config.description(), "");
        assert_eq!(config.source(), PathBuf::from("src/"));
        assert!(config.authors().is_empty());
    }
}
