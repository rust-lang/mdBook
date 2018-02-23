//! This program removes all forms of emphasis from the markdown of the book.
extern crate mdbook;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;

use mdbook::errors::{Error, Result};
use mdbook::MDBook;
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, Parser, Tag};
use pulldown_cmark_to_cmark::fmt::cmark;

use std::ffi::OsString;
use std::env::{args, args_os};
use std::process;

struct Deemphasize;

impl Preprocessor for Deemphasize {
    fn name(&self) -> &str {
        "md-links-to-html-links"
    }

    fn run(&self, _ctx: &PreprocessorContext, book: &mut Book) -> Result<()> {
        eprintln!("Running '{}' preprocessor", self.name());
        let mut res: Option<_> = None;
        let mut num_removed_items = 0;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }
            if let BookItem::Chapter(ref mut chapter) = *item {
                eprintln!("{}: processing chapter '{}'", self.name(), chapter.name);
                res = Some(
                    match Deemphasize::remove_emphasis(&mut num_removed_items, chapter) {
                        Ok(md) => {
                            chapter.content = md;
                            Ok(())
                        }
                        Err(err) => Err(err),
                    },
                );
            }
        });
        eprintln!(
            "{}: removed {} events from markdown stream.",
            self.name(),
            num_removed_items
        );
        match res {
            Some(res) => res,
            None => Ok(()),
        }
    }
}

fn do_it(book: OsString) -> Result<()> {
    let mut book = MDBook::load(book)?;
    book.with_preprecessor(Deemphasize);
    book.build()
}

fn main() {
    if args_os().count() != 2 {
        eprintln!("USAGE: {} <book>", args().next().expect("executable"));
        return;
    }
    if let Err(e) = do_it(args_os().skip(1).next().expect("one argument")) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

impl Deemphasize {
    fn remove_emphasis(num_removed_items: &mut i32, chapter: &mut Chapter) -> Result<String> {
        let mut buf = String::with_capacity(chapter.content.len());
        let events = Parser::new(&chapter.content).filter(|e| {
            let should_keep = match *e {
                Event::Start(Tag::Emphasis)
                | Event::Start(Tag::Strong)
                | Event::End(Tag::Emphasis)
                | Event::End(Tag::Strong) => false,
                _ => true,
            };
            if !should_keep {
                *num_removed_items += 1;
            }
            should_keep
        });
        cmark(events, &mut buf, None)
            .map(|_| buf)
            .map_err(|err| Error::from(format!("Markdown serialization failed: {}", err)))
    }
}
