use renderer::Renderer;
use book::MDBook;
use book::bookitem::BookItem;

use std::error::Error;
use std::process::Command;

pub struct Pandoc;

impl Pandoc {
  pub fn new() -> Pandoc {
    Pandoc
  }
}

impl Renderer for Pandoc {
    fn render(&self, book: &MDBook) -> Result<(), Box<Error>> {
        let mut paths = vec!();

        for item in book.iter() {
            match *item {
                BookItem::Chapter(_, ref ch) => {
                    paths.push(book.get_src().join(&ch.path).into_os_string());
                },
                _ => println!("FIXME: don't understand this kind of BookItem")
            }
        }

        let output = Command::new("pandoc")
            .arg("-S")
            .arg("-osample.epub")
            .args(&paths)
            .output();

        match output {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e))
        }
        // FIXME: why doesn't this work
        // output.map(|_| ()).map_err(|e| Box::new(e))
    }
}