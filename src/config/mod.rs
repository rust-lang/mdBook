pub mod bookconfig;
pub mod htmlconfig;
pub mod tomlconfig;
pub mod jsonconfig;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use errors::*;

// Re-export the config structs
pub use self::bookconfig::BookConfig;
pub use self::htmlconfig::HtmlConfig;
pub use self::tomlconfig::TomlConfig;

/// Parses the `book.json` file (if it exists) to extract
/// the configuration parameters.
/// The `book.json` file should be in the root directory of the book.
/// The root directory is the one specified when creating a new `MDBook`

pub fn read_config<P: AsRef<Path>>(root: P) -> Result<BookConfig> {
    let root = root.as_ref();
    let toml = root.join("book.toml");
    let json = root.join("book.json");

    if toml.exists() {
        let mut file = File::open(toml)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let cfg = TomlConfig::from_toml(&content)?;
        Ok(BookConfig::from_tomlconfig(root, cfg))
    } else if json.exists() {
        warn!("The JSON configuration file is deprecated, please use the TOML configuration.");
        let mut file = File::open(json)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let jason = jsonconfig::JsonConfig::from_json(&content)?;
        Ok(BookConfig::from_jsonconfig(root, jason))
    } else {
        Err(Error::from("No config file found"))
    }
}
