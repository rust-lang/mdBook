extern crate rustc_serialize;

use self::rustc_serialize::json::{Json, ToJson};
use std::path::PathBuf;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct BookItem {
    pub name: String,
    pub path: PathBuf,
    pub sub_items: Vec<BookItem>,
    spacer: bool,
}

#[derive(Debug, Clone)]
pub struct BookItems<'a> {
    pub items: &'a [BookItem],
    pub current_index: usize,
    pub stack: Vec<(&'a [BookItem], usize)>,
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

    fn _spacer() -> Self {
        BookItem {
            name: String::from("SPACER"),
            path: PathBuf::new(),
            sub_items: vec![],
            spacer: true,
        }
    }
}


impl ToJson for BookItem {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("name".to_string(), self.name.to_json());
        m.insert("path".to_string(),self.path.to_str()
            .expect("Json conversion failed for path").to_json()
        );
        m.to_json()
    }
}



// Shamelessly copied from Rustbook
// (https://github.com/rust-lang/rust/blob/master/src/rustbook/book.rs)
impl<'a> Iterator for BookItems<'a> {
    type Item = (String, &'a BookItem);

    fn next(&mut self) -> Option<(String, &'a BookItem)> {
        loop {
            if self.current_index >= self.items.len() {
                match self.stack.pop() {
                    None => return None,
                    Some((parent_items, parent_idx)) => {
                        self.items = parent_items;
                        self.current_index = parent_idx + 1;
                    }
                }
            } else {
                let cur = self.items.get(self.current_index).unwrap();

                let mut section = "".to_string();
                for &(_, idx) in &self.stack {
                    section.push_str(&(idx + 1).to_string()[..]);
                    section.push('.');
                }
                section.push_str(&(self.current_index + 1).to_string()[..]);
                section.push('.');

                self.stack.push((self.items, self.current_index));
                self.items = &cur.sub_items[..];
                self.current_index = 0;
                return Some((section, cur))
            }
        }
    }
}
