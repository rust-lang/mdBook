//! This is a demonstration of an mdBook preprocessor which parses markdown
//! and removes any instances of emphasis.

use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::io;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("supports") => {
            // Supports all renderers.
            return;
        }
        Some(arg) => {
            eprintln!("unknown argument: {arg}");
            std::process::exit(1);
        }
        None => {}
    }

    if let Err(e) = handle_preprocessing() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

struct RemoveEmphasis;

impl Preprocessor for RemoveEmphasis {
    fn name(&self) -> &str {
        "remove-emphasis"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut total = 0;
        book.for_each_mut(|item| {
            let BookItem::Chapter(ch) = item else {
                return;
            };
            if ch.is_draft_chapter() {
                return;
            }
            match remove_emphasis(&mut total, ch) {
                Ok(s) => ch.content = s,
                Err(e) => eprintln!("failed to process chapter: {e:?}"),
            }
        });
        eprintln!("removed {total} emphasis");
        Ok(book)
    }
}

// ANCHOR: remove_emphasis
fn remove_emphasis(num_removed_items: &mut usize, chapter: &mut Chapter) -> Result<String> {
    let mut buf = String::with_capacity(chapter.content.len());

    let events = Parser::new(&chapter.content).filter(|e| match e {
        Event::Start(Tag::Emphasis) | Event::Start(Tag::Strong) => {
            *num_removed_items += 1;
            false
        }
        Event::End(TagEnd::Emphasis) | Event::End(TagEnd::Strong) => false,
        _ => true,
    });

    Ok(pulldown_cmark_to_cmark::cmark(events, &mut buf).map(|_| buf)?)
}
// ANCHOR_END: remove_emphasis

pub fn handle_preprocessing() -> Result<()> {
    let pre = RemoveEmphasis;
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
