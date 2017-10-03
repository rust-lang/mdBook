extern crate serde_json;
use std::path::PathBuf;
use errors::*;

/// The JSON configuration is **deprecated** and will be removed in the near future.
/// Please migrate to the TOML configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonConfig {
    pub src: Option<PathBuf>,
    pub dest: Option<PathBuf>,

    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,

    pub theme_path: Option<PathBuf>,
    pub google_analytics: Option<String>,
}


/// Returns a `JsonConfig` from a JSON string
///
/// ```
/// # use mdbook::config::jsonconfig::JsonConfig;
/// # use std::path::PathBuf;
/// let json = r#"{
///     "title": "Some title",
///     "dest": "htmlbook"
/// }"#;
///
/// let config = JsonConfig::from_json(&json).expect("Should parse correctly");
/// assert_eq!(config.title, Some(String::from("Some title")));
/// assert_eq!(config.dest, Some(PathBuf::from("htmlbook")));
/// ```
impl JsonConfig {
    pub fn from_json(input: &str) -> Result<Self> {
        let config: JsonConfig = serde_json::from_str(input).chain_err(|| "Could not parse JSON")?;

        Ok(config)
    }
}
