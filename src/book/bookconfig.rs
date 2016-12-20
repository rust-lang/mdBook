use serde_json;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

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

        // If the file does not exist, return early
        let mut config_file = match File::open(root.join("book.json")) {
            Ok(f) => f,
            Err(_) => {
                debug!("[*]: Failed to open {:?}", root.join("book.json"));
                return self;
            },
        };

        debug!("[*]: Reading config");
        let mut data = String::new();

        // Just return if an error occured.
        // I would like to propagate the error, but I have to return `&self`
        if let Err(_) = config_file.read_to_string(&mut data) {
            return self;
        }

        // Convert to JSON
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&data) {
            // Extract data

            let config = config.as_object().unwrap();

            debug!("[*]: Extracting data from config");
            // Title, author, description
            if let Some(a) = config.get("title") {
                self.title = a.to_string().replace("\"", "")
            }
            if let Some(a) = config.get("author") {
                self.author = a.to_string().replace("\"", "")
            }
            if let Some(a) = config.get("description") {
                self.description = a.to_string().replace("\"", "")
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
