use std::path::PathBuf;

pub struct BookItem {
    name: String,
    path: PathBuf,
    sub_items: Vec<BookItem>,
}


impl BookItem {

    fn new(name: String, path: PathBuf) -> Self {

        BookItem {
            name: name,
            path: path,
            sub_items: vec![],
        }
    }

}
