//! This program removes all forms of emphasis from the markdown of the book.
extern crate mdbook;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;

use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::MDBook;
use pulldown_cmark::{Event, Parser, Tag};
use pulldown_cmark_to_cmark::fmt::cmark;

use std::env::{args, args_os};
use std::ffi::OsString;
use std::process;

const NAME: &str = "md-links-to-html-links";

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

struct Deemphasize;

impl Preprocessor for Deemphasize {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        eprintln!("Running '{}' preprocessor", self.name());
        let mut num_removed_items = 0;

        process(&mut book.sections, &mut num_removed_items)?;

        eprintln!(
            "{}: removed {} events from markdown stream.",
            self.name(),
            num_removed_items
        );

        Ok(book)
    }
}

fn process<'a, I>(items: I, num_removed_items: &mut usize) -> Result<()>
where
    I: IntoIterator<Item = &'a mut BookItem> + 'a,
{
    for item in items {
        if let BookItem::Chapter(ref mut chapter) = *item {
            eprintln!("{}: processing chapter '{}'", NAME, chapter.name);

            let md = remove_emphasis(num_removed_items, chapter)?;
            chapter.content = md;
        }
    }

    Ok(())
}

fn remove_emphasis(
    num_removed_items: &mut usize,
    chapter: &mut Chapter,
) -> Result<String> {
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

    cmark(events, &mut buf, None).map(|_| buf).map_err(|err| {
        Error::from(format!("Markdown serialization failed: {}", err))
    })
}
