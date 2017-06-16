use std::path::PathBuf;
use std::collections::HashMap;

pub type RendererConfig = HashMap<String, String>;

/// Configuration struct for a `mdbook` project directory.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct Config {
    source: PathBuf,
    #[serde(rename = "renderer")]
    renderers: HashMap<String, RendererConfig>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            source: PathBuf::from("src"),
            renderers: Default::default(),
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
}
