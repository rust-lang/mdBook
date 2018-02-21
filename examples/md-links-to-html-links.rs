extern crate mdbook;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;

use mdbook::errors::Error;
use mdbook::MDBook;
use mdbook::book::Book;
use mdbook::preprocess::{PreprocessorContext, Preprocessor};

use std::ffi::OsString;
use std::env::{args_os, args};

struct ProcessLinks;

impl Preprocessor for ProcessLinks {
    fn name(&self) -> &str {
        "md-links-to-html-links"
    }

    fn run(&self, ctx: &PreprocessorContext, book: &mut Book) -> ::std::result::Result<(), Error> {
        eprintln!("Running '{}' preprocessor", self.name());
        Ok(())
    }
}

fn do_it(book: OsString) -> ::std::result::Result<(), Error> {
    let mut book = MDBook::load(book)?;
    book.with_preprecessor(ProcessLinks);
    Ok(())
}

fn main() {
    if args_os().count() != 2 {
        eprintln!("USAGE: {} <book>", args().next().expect("executable"));
        return
    }
    if let Err(e) = do_it(args_os()
                        .skip(1)
                        .next()
                        .expect("one argument")) {
        eprintln!("{}", e);
    }
}

