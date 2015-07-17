use std::path::PathBuf;

pub struct BookItem {
    pub name: String,
    pub path: PathBuf,
    pub sub_items: Vec<BookItem>,
    spacer: bool,
}

pub enum ItemType {
    Pre,
    Chapter,
    Post
}

impl BookItem {

    pub fn new(name: String, path: PathBuf) -> Self {

        BookItem {
            name: name,
            path: path,
            sub_items: vec![],
            spacer: false,
        }
    }

    pub fn spacer() -> Self {
        BookItem {
            name: String::from("SPACER"),
            path: PathBuf::new(),
            sub_items: vec![],
            spacer: true,
        }
    }

    fn push(&mut self, item: BookItem) {
        self.sub_items.push(item);
    }

}
