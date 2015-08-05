extern crate rustc_serialize;
use self::rustc_serialize::json::Json;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BookConfig {
    pub title: String,
    pub author: String,
    dest: PathBuf,
    src: PathBuf,
    pub indent_spaces: i32,
    multilingual: bool,
}


impl BookConfig {
    pub fn new() -> Self {
        BookConfig {
            title: String::new(),
            author: String::new(),
            dest: PathBuf::from("book"),
            src: PathBuf::from("src"),
            indent_spaces: 4,
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
                return self
            },
        };

        debug!("[*]: Reading config");
        let mut data = String::new();
        config_file.read_to_string(&mut data).unwrap();

        // Convert to JSON
        let config = Json::from_str(&data).unwrap();

        // Extract data

        debug!("[*]: Extracting data from config");
        // Title & author
        if let Some(a) = config.find_path(&["title"]) { self.title = a.to_string().replace("\"", "") }
        if let Some(a) = config.find_path(&["author"]) { self.author = a.to_string().replace("\"", "") }

        // Destination
        if let Some(a) = config.find_path(&["dest"]) {
            let dest = PathBuf::from(&a.to_string().replace("\"", ""));

            // If path is relative make it absolute from the parent directory of src
            if dest.is_relative() {
                let dest = &self.src().parent().unwrap().join(&dest);
                self.set_dest(dest);
            }
        }

        self
    }

    pub fn dest(&self) -> &Path {
        &self.dest
    }

    pub fn set_dest(&mut self, dest: &Path) -> &mut Self {
        self.dest = dest.to_owned();
        self
    }

    pub fn src(&self) -> &Path {
        &self.src
    }

    pub fn set_src(&mut self, src: &Path) -> &mut Self {
        self.src = src.to_owned();
        self
    }

}
