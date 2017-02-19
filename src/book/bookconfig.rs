extern crate toml;

use std::process::exit;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::str::FromStr;
use serde_json;

#[derive(Debug, Clone)]
pub struct BookConfig {
    root: PathBuf,
    pub dest: PathBuf,
    pub src: PathBuf,
    pub theme_path: PathBuf,

    pub title: String,
    pub author: String,
    pub description: String,

    pub indent_spaces: i32,
    multilingual: bool,
}

impl BookConfig {
    pub fn new(root: &Path) -> Self {
        BookConfig {
            root: root.to_owned(),
            dest: root.join("book"),
            src: root.join("src"),
            theme_path: root.join("theme"),

            title: String::new(),
            author: String::new(),
            description: String::new(),

            indent_spaces: 4, // indentation used for SUMMARY.md
            multilingual: false,
        }
    }

    pub fn read_config(&mut self, root: &Path) -> &mut Self {

        debug!("[fn]: read_config");

        let read_file = |path: PathBuf| -> String {
            let mut data = String::new();
            let mut f: File = match File::open(&path) {
                Ok(x) => x,
                Err(_) => {
                    error!("[*]: Failed to open {:?}", &path);
                    exit(2);
                }
            };
            if f.read_to_string(&mut data).is_err() {
                error!("[*]: Failed to read {:?}", &path);
                exit(2);
            }
            data
        };

        // Read book.toml or book.json if exists

        if root.join("book.toml").exists() {

            debug!("[*]: Reading config");
            let data = read_file(root.join("book.toml"));
            self.parse_from_toml_string(&data);

        } else if root.join("book.json").exists() {

            debug!("[*]: Reading config");
            let data = read_file(root.join("book.json"));
            self.parse_from_json_string(&data);

        } else {
            debug!("[*]: No book.toml or book.json was found, using defaults.");
        }

        self
    }

    pub fn parse_from_toml_string(&mut self, data: &str) -> &mut Self {
        let config = match toml::from_str(data) {
            Ok(x) => {x},
            Err(e) => {
                error!("[*]: Toml parse errors in book.toml: {:?}", e);
                exit(2);
            }
        };

        self.parse_from_btreemap(&config);

        self
    }

    /// Parses the string to JSON and converts it to BTreeMap<String, toml::Value>.
    pub fn parse_from_json_string(&mut self, data: &str) -> &mut Self {

        let c: serde_json::Value = match serde_json::from_str(data) {
            Ok(x) => x,
            Err(e) => {
                error!("[*]: JSON parse errors in book.json: {:?}", e);
                exit(2);
            }
        };

        let config = json_object_to_btreemap(c.as_object().unwrap());
        self.parse_from_btreemap(&config);

        self
    }

    pub fn parse_from_btreemap(&mut self, config: &BTreeMap<String, toml::Value>) -> &mut Self {

        // Title, author, description
        if let Some(a) = config.get("title") {
            self.title = a.to_string().replace("\"", "");
        }
        if let Some(a) = config.get("author") {
            self.author = a.to_string().replace("\"", "");
        }
        if let Some(a) = config.get("description") {
            self.description = a.to_string().replace("\"", "");
        }

        // Destination folder
        if let Some(a) = config.get("dest") {
            let mut dest = PathBuf::from(&a.to_string().replace("\"", ""));

            // If path is relative make it absolute from the parent directory of src
            if dest.is_relative() {
                dest = self.get_root().join(&dest);
            }
            self.set_dest(&dest);
        }

        // Source folder
        if let Some(a) = config.get("src") {
            let mut src = PathBuf::from(&a.to_string().replace("\"", ""));
            if src.is_relative() {
                src = self.get_root().join(&src);
            }
            self.set_src(&src);
        }

        // Theme path folder
        if let Some(a) = config.get("theme_path") {
            let mut theme_path = PathBuf::from(&a.to_string().replace("\"", ""));
            if theme_path.is_relative() {
                theme_path = self.get_root().join(&theme_path);
            }
            self.set_theme_path(&theme_path);
        }

        self
    }

    pub fn get_root(&self) -> &Path {
        &self.root
    }

    pub fn set_root(&mut self, root: &Path) -> &mut Self {
        self.root = root.to_owned();
        self
    }

    pub fn get_dest(&self) -> &Path {
        &self.dest
    }

    pub fn set_dest(&mut self, dest: &Path) -> &mut Self {
        self.dest = dest.to_owned();
        self
    }

    pub fn get_src(&self) -> &Path {
        &self.src
    }

    pub fn set_src(&mut self, src: &Path) -> &mut Self {
        self.src = src.to_owned();
        self
    }

    pub fn get_theme_path(&self) -> &Path {
        &self.theme_path
    }

    pub fn set_theme_path(&mut self, theme_path: &Path) -> &mut Self {
        self.theme_path = theme_path.to_owned();
        self
    }
}

pub fn json_object_to_btreemap(json: &serde_json::Map<String, serde_json::Value>) -> BTreeMap<String, toml::Value> {
    let mut config: BTreeMap<String, toml::Value> = BTreeMap::new();

    for (key, value) in json.iter() {
        config.insert(
            String::from_str(key).unwrap(),
            json_value_to_toml_value(value.to_owned())
        );
    }

    config
}

pub fn json_value_to_toml_value(json: serde_json::Value) -> toml::Value {
    match json {
        serde_json::Value::Null => toml::Value::String("".to_string()),
        serde_json::Value::Bool(x) => toml::Value::Boolean(x),
        serde_json::Value::Number(ref x) if x.is_i64() => toml::Value::Integer(x.as_i64().unwrap()),
        serde_json::Value::Number(ref x) if x.is_u64() => toml::Value::Integer(x.as_i64().unwrap()),
        serde_json::Value::Number(x) => toml::Value::Float(x.as_f64().unwrap()),
        serde_json::Value::String(x) => toml::Value::String(x),
        serde_json::Value::Array(x) => {
            toml::Value::Array(x.iter().map(|v| json_value_to_toml_value(v.to_owned())).collect())
        },
        serde_json::Value::Object(x) => {
            toml::Value::Table(json_object_to_btreemap(&x))
        },
    }
}
