//! Configuration management.

use std::path::{Path, PathBuf};
use std::fs::File;
use std::collections::HashMap;
use std::io::Read;
use std::ops::Deref;

use toml;
use errors::*;


pub type RendererConfig = HashMap<String, String>;

/// Try to load the config file from the provided directory, automatically
/// detecting the supported formats.
pub fn load_config<P: AsRef<Path>>(root: P) -> Result<Config> {
    // TODO: add a `Config::from_json()` call here if the toml one fails
    let toml_path = root.as_ref().join("book.toml");
    debug!("[*] Attempting to load the config file from {}", toml_path.display());

    Config::from_toml(&toml_path)
}

/// Configuration struct for a `mdbook` project directory.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct Config {
    source: PathBuf,
    #[serde(rename = "renderer")]
    renderers: HashMap<String, RendererConfig>,
    title: String,
    description: Option<String>,
    author: Option<String>,
}

impl Config {
    fn from_toml<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path = path.as_ref();
        if !path.exists() {
            bail!("The configuration file doesn't exist");
        }

        if !path.is_file() {
            bail!("The provided path doesn't point to a file");
        }

        let mut contents = String::new();
        File::open(&path)
            .chain_err(|| "Couldn't open the config file for reading")?
            .read_to_string(&mut contents)?;

        toml::from_str(&contents).chain_err(|| "Config parsing failed")
    }

    pub fn source_directory(&self) -> &Path {
        &self.source
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn author(&self) -> Option<&str> {
        self.author.as_ref().map(Deref::deref)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            source: PathBuf::from("src"),
            renderers: Default::default(),
            author: None,
            description: None,
            title: "Example Book".to_owned(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn deserialize_basic_config() {
        let src = r#"
        [renderer.html]
        "#;

        let got: Config = toml::from_str(src).unwrap();
        println!("{:#?}", got);

        assert_eq!(got.source, PathBuf::from("src"));
        assert!(got.renderers.contains_key("html"));
    }

    /// This test will read from the `book.toml` in the `book-example`, making
    /// sure that the `Config::from_toml()` method works and that we maintain
    /// backwards compatibility.
    #[test]
    fn read_a_working_config_toml() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("book-example")
            .join("book.toml");
        println!("{}", path.display());

        let got = Config::from_toml(&path).unwrap();

        assert_eq!(got.title, "mdBook Documentation");
        assert_eq!(
            got.description,
            Some("Create book from markdown files. Like Gitbook but implemented in Rust".to_string())
        );
        assert_eq!(got.author, Some("Mathieu David".to_string()));
    }
}
