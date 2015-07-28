use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BookConfig {
    pub title: String,
    pub author: String,
    dest: PathBuf,
    src: PathBuf,
    indent_spaces: i32,
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
