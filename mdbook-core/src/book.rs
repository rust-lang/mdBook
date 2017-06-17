use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use errors::*;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    sections: Vec<BookItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BookItem {
    Chapter(String, Chapter),
    Affix(Chapter),
    Spacer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    /// The chapter name as specified in the `SUMMARY.md`.
    pub name: String,
    /// The file's location relative to the project root.
    pub path: PathBuf,
    /// The chapter's raw text.
    pub contents: String,
    /// Any sub-items in the chapter.
    pub items: Vec<BookItem>,
}

impl Chapter {
    pub fn new(name: String, path: PathBuf) -> Result<Chapter> {
        let mut contents = String::new();
        File::open(&path)?.read_to_string(&mut contents)?;

        Ok(Chapter {
            name: name,
            path: path,
            contents: contents,
            items: Vec::new(),
        })
    }
}
