use std::path::{PathBuf, Path};

use super::HtmlConfig;
use super::tomlconfig::TomlConfig;

/// Configuration struct containing all the configuration options available in mdBook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BookConfig {
    root: PathBuf,
    source: PathBuf,

    title: String,
    authors: Vec<String>,
    description: String,

    multilingual: bool,
    indent_spaces: i32,

    html_config: Option<HtmlConfig>,
}

impl BookConfig {
    /// Creates a new `BookConfig` struct with as root path the path given as parameter.
    /// The source directory is `root/src` and the destination for the rendered book is `root/book`.
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use mdbook::config::{BookConfig, HtmlConfig};
    /// #
    /// let root = PathBuf::from("directory/to/my/book");
    /// let config = BookConfig::new(&root);
    ///
    /// assert_eq!(config.get_root(), &root);
    /// assert_eq!(config.get_source(), PathBuf::from("directory/to/my/book/src"));
    /// assert_eq!(config.get_html_config(), Some(&HtmlConfig::new(PathBuf::from("directory/to/my/book"))));
    /// ```
    pub fn new<T: Into<PathBuf>>(root: T) -> Self {
        let root: PathBuf = root.into();
        let htmlconfig = HtmlConfig::new(&root);

        BookConfig {
            root: root.clone(),
            source: root.join("src"),

            title: String::new(),
            authors: Vec::new(),
            description: String::new(),

            multilingual: false,
            indent_spaces: 4,

            html_config: Some(htmlconfig),
        }
    }

    /// Builder method to set the source directory
    pub fn with_source<T: Into<PathBuf>>(mut self, source: T) -> Self {
        self.source = source.into();
        self
    }

    /// Builder method to set the book's title
    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = title.into();
        self
    }

    /// Builder method to set the book's description
    pub fn with_description<T: Into<String>>(mut self, description: T) -> Self {
        self.description = description.into();
        self
    }

    /// Builder method to set the book's authors
    pub fn with_authors<T: Into<Vec<String>>>(mut self, authors: T) -> Self {
        self.authors = authors.into();
        self
    }

    pub fn from_tomlconfig<T: Into<PathBuf>>(root: T, tomlconfig: TomlConfig) -> Self {
        let root = root.into();
        let mut config = BookConfig::new(&root);

        if let Some(s) = tomlconfig.source {
            config.set_source(s);
        }

        if let Some(t) = tomlconfig.title {
            config.set_title(t);
        }

        if let Some(d) = tomlconfig.description {
            config.set_description(d);
        }

        if let Some(a) = tomlconfig.authors {
            config.set_authors(a);
        }

        if let Some(a) = tomlconfig.author {
            config.set_authors(vec![a]);
        }

        if let Some(tomlhtmlconfig) = tomlconfig.output.and_then(|o| o.html) {
            let mut htmlconfig = config.get_mut_html_config().expect("We just created a new config and it creates a default HtmlConfig");
            htmlconfig.fill_from_tomlconfig(&root, tomlhtmlconfig);
        }
        
        config
    }

    pub fn fill_from_tomlconfig(&mut self, tomlconfig: TomlConfig) -> &mut Self {

        if let Some(s) = tomlconfig.source {
            self.set_source(s);
        }

        if let Some(t) = tomlconfig.title {
            self.set_title(t);
        }

        if let Some(d) = tomlconfig.description {
            self.set_description(d);
        }

        if let Some(a) = tomlconfig.authors {
            self.set_authors(a);
        }

        if let Some(a) = tomlconfig.author {
            self.set_authors(vec![a]);
        }

        if let Some(tomlhtmlconfig) = tomlconfig.output.and_then(|o| o.html) {
            let root = self.root.clone();
            if let Some(htmlconfig) = self.get_mut_html_config() {
                htmlconfig.fill_from_tomlconfig(root, tomlhtmlconfig);
            }
        }
        
        self
    }

    pub fn set_root<T: Into<PathBuf>>(&mut self, root: T) -> &mut Self {
        self.root = root.into();
        self
    }

    pub fn get_root(&self) -> &Path {
        &self.root
    }
    
    pub fn set_source<T: Into<PathBuf>>(&mut self, source: T) -> &mut Self {
        let mut source = source.into();
        
        // If the source path is relative, start with the root path
        if source.is_relative() {
            source = self.root.join(source);
        }

        self.source = source; 
        self
    }

    pub fn get_source(&self) -> &Path {
        &self.source
    }

    pub fn set_title<T: Into<String>>(&mut self, title: T) -> &mut Self {
        self.title = title.into();
        self
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_description<T: Into<String>>(&mut self, description: T) -> &mut Self {
        self.description = description.into();
        self
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn set_authors<T: Into<Vec<String>>>(&mut self, authors: T) -> &mut Self {
        self.authors = authors.into();
        self
    }

    /// Returns the authors of the book as specified in the configuration file
    pub fn get_authors(&self) -> &[String] {
        self.authors.as_slice()
    }

    pub fn  set_html_config(&mut self, htmlconfig: HtmlConfig) -> &mut Self {
        self.html_config = Some(htmlconfig);
        self
    }

    /// Returns the configuration for the HTML renderer or None of there isn't any
    pub fn get_html_config(&self) -> Option<&HtmlConfig> {
        self.html_config.as_ref()
    }

    pub fn get_mut_html_config(&mut self) -> Option<&mut HtmlConfig> {
        self.html_config.as_mut()
    }
}
