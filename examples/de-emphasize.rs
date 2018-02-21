extern crate mdbook;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;

// This program removes all forms of emphasis from the markdown of the book.

use mdbook::errors::Error;
use mdbook::MDBook;
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, Parser, Tag};
use pulldown_cmark_to_cmark::fmt::cmark;

use std::ffi::OsString;
use std::env::{args, args_os};
use std::process;

struct ProcessLinks;

impl Preprocessor for ProcessLinks {
    fn name(&self) -> &str {
        "md-links-to-html-links"
    }

    fn run(&self, _ctx: &PreprocessorContext, book: &mut Book) -> ::std::result::Result<(), Error> {
        eprintln!("Running '{}' preprocessor", self.name());
        let mut res: Option<_> = None;
        let mut num_removed_items = 0;
        book.for_each_mut(|item: &mut BookItem| {
            if let &Some(Err(_)) = &res {
                return;
            }
            if let BookItem::Chapter(ref mut chapter) = *item {
                eprintln!("{}: processing chapter '{}'", self.name(), chapter.name);
                let mut buf = String::with_capacity(chapter.content.len());
                res = Some({
                    let events = Parser::new(&chapter.content).filter(|e| {
                        let should_keep = match *e {
                            Event::Start(Tag::Emphasis)
                            | Event::Start(Tag::Strong)
                            | Event::End(Tag::Emphasis)
                            | Event::End(Tag::Strong) => false,
                            _ => true,
                        };
                        if !should_keep {
                            num_removed_items += 1;
                        }
                        should_keep
                    });
                    cmark(events, &mut buf, None)
                });
                if let &Some(Ok(_)) = &res {
                    chapter.content = buf;
                }
            }
        });
        eprintln!(
            "{}: removed {} events from markdown stream.",
            self.name(),
            num_removed_items
        );
        if let Some(Err(err)) = res {
            Err(Error::from(format!(
                "Markdown serialization failed: {}",
                err
            )))
        } else {
            Ok(())
        }
    }
}

fn do_it(book: OsString) -> ::std::result::Result<(), Error> {
    let mut book = MDBook::load(book)?;
    book.with_preprecessor(ProcessLinks);
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
