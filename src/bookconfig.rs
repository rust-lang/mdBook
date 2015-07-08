use std::path::PathBuf;

pub struct BookConfig {
    dest: PathBuf,
    src: PathBuf,
    multilingual: bool,
}


impl BookConfig {
    pub fn new() -> Self {
        BookConfig {
            dest: PathBuf::from("book"),
            src: PathBuf::from("src"),
            multilingual: false,
        }
    }

    pub fn dest(&self) -> PathBuf {
        self.dest.clone()
    }

    pub fn set_dest(&mut self, dest: PathBuf) {

        // dest has to be relative to the path in MDBook,
        // we check if the path is relative, otherwhise we truncate
        if dest.is_relative() {
            self.dest = dest
        } else {
            self.dest = PathBuf::from(dest.file_name().unwrap())
        }
    }

    pub fn src(&self) -> PathBuf {
        self.src.clone()
    }

    pub fn set_src(&mut self, src: PathBuf) {

        // src has to be relative to the path in MDBook,
        // we check if the path is relative, otherwhise we truncate
        if src.is_relative() {
            self.src = src
        } else {
            self.src = PathBuf::from(src.file_name().unwrap())
        }
    }
}
