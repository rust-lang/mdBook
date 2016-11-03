use serde_json;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BookConfig {
    pub title: String,
    pub author: String,
    pub description: String,
    root: PathBuf,
    pub dest: PathBuf,
    pub src: PathBuf,
    pub indent_spaces: i32,
    multilingual: bool,
}


impl BookConfig {
    pub fn new(root: &Path) -> Self {
        BookConfig {
            title: String::new(),
            author: String::new(),
            description: String::new(),
            root: root.to_owned(),
            dest: root.join("book"),
            src: root.join("src"),
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

            // Destination
            if let Some(a) = config.get("dest") {
                let dest = PathBuf::from(&a.to_string().replace("\"", ""));

                // If path is relative make it absolute from the parent directory of src
                match dest.is_relative() {
                    true => {
                        let dest = self.get_root().join(&dest).to_owned();
                        self.set_dest(&dest);
                    },
                    false => {
                        self.set_dest(&dest);
                    },
                }
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
}
