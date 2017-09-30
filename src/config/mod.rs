use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use toml::{self, Value};

use errors::*;

pub mod bookconfig;
pub mod htmlconfig;
pub mod playpenconfig;
pub mod tomlconfig;
pub mod jsonconfig;

// Re-export the config structs
pub use self::bookconfig::BookConfig;
pub use self::htmlconfig::HtmlConfig;
pub use self::playpenconfig::PlaypenConfig;
pub use self::tomlconfig::TomlConfig;


/// The overall configuration object for MDBook.
#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Metadata about the book.
    pub book: BookConfig_,
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
        File::open(config_file)
            .chain_err(|| "Unable to open the configuration file")?
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
    pub fn html_config(&self) -> Option<HtmlConfig_> {
        self.output
            .get("html")
            .and_then(|value| HtmlConfig_::from_toml(value).ok())
    }
}


/// Configuration options which are specific to the book and required for
/// loading it from disk.
#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct BookConfig_ {
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

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct HtmlConfig_ {
    pub theme: Option<PathBuf>,
    pub curly_quotes: bool,
    pub mathjax_support: bool,
    pub google_analytics: Option<String>,
    pub additional_css: Vec<PathBuf>,
    pub additional_js: Vec<PathBuf>,
    pub playpen: Playpen,
}

impl HtmlConfig_ {
    pub fn from_toml(value: &Value) -> Result<HtmlConfig_> {
        value
            .clone()
            .try_into()
            .chain_err(|| "Unable to deserialize the HTML config")
    }
}

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Playpen;
