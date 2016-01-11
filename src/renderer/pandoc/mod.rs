use renderer::Renderer;
use book::MDBook;
use book::bookitem::BookItem;

use std::error::Error;

enum PandocOutput {
    Epub,
    Pdf
}

pub struct Pandoc {
    output: PandocOutput
}

impl Pandoc {
  pub fn new() -> Pandoc {
    Pandoc { output: PandocOutput::Epub }
  }
}

impl Renderer for Pandoc {
    fn render(&self, book: &MDBook) -> Result<(), Box<Error>> {

        for item in book.iter() {
            match *item {
                BookItem::Chapter(ref title, _) => {
                    println!("{}", title)
                },
                _ => println!("Something I don't understand")
            }
        }

        Ok(())
    }
}