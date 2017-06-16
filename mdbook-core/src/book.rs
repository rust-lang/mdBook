use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    sections: Vec<BookItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BookItem {
    Chapter(Chapter),
    Spacer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    /// The chapter name as specified in the `SUMMARY.md`.
    name: String,
    /// The file's location relative to the project root.
    path: PathBuf,
    /// The chapter's raw text.
    contents: String,
    /// Any sub-items in the chapter.
    items: Vec<BookItem>,
}
