use serde::{Serialize, Serializer};
use std::path::PathBuf;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum BookItem {
    Chapter(String, Chapter), // String = section
    Affix(Chapter),
    Spacer,
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub name: String,
    pub path: PathBuf,
    pub sub_items: Vec<BookItem>,
}

#[derive(Debug, Clone)]
pub struct BookItems<'a> {
    pub items: &'a [BookItem],
    pub current_index: usize,
    pub stack: Vec<(&'a [BookItem], usize)>,
}


impl Chapter {
    pub fn new(name: String, path: PathBuf) -> Self {

        Chapter {
            name: name,
            path: path,
            sub_items: vec![],
        }
    }
}


impl Serialize for Chapter {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        let mut state = try!(serializer.serialize_struct("Chapter", 2));
        try!(serializer.serialize_struct_elt(&mut state, "name", self.name.clone()));
        try!(serializer.serialize_struct_elt(&mut state, "path", self.path.clone()));
        serializer.serialize_struct_end(state)
    }
}



// Shamelessly copied from Rustbook
// (https://github.com/rust-lang/rust/blob/master/src/rustbook/book.rs)
impl<'a> Iterator for BookItems<'a> {
    type Item = &'a BookItem;

    fn next(&mut self) -> Option<&'a BookItem> {
        loop {
            if self.current_index >= self.items.len() {
                match self.stack.pop() {
                    None => return None,
                    Some((parent_items, parent_idx)) => {
                        self.items = parent_items;
                        self.current_index = parent_idx + 1;
                    },
                }
            } else {
                let cur = self.items.get(self.current_index).unwrap();

                match *cur {
                    BookItem::Chapter(_, ref ch) | BookItem::Affix(ref ch) => {
                        self.stack.push((self.items, self.current_index));
                        self.items = &ch.sub_items[..];
                        self.current_index = 0;
                    },
                    BookItem::Spacer => {
                        self.current_index += 1;
                    },
                }

                return Some(cur);
            }
        }
    }
}
