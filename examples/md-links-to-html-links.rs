extern crate mdbook;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
#[macro_use]
extern crate quicli;

use mdbook::errors::Error;
use mdbook::MDBook;
use mdbook::book::Book;
use mdbook::preprocess::{PreprocessorContext, Preprocessor};
use quicli::prelude::*;

#[derive(Debug, StructOpt)]
struct Cli {
    book: String,

    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    verbosity: u8,
}

struct ProcessLinks;

impl Preprocessor for ProcessLinks {
    fn name(&self) -> &str {
        "md-links-to-html-links"
    }

    fn run(&self, ctx: &PreprocessorContext, book: &mut Book) -> ::std::result::Result<(), Error> {
        info!("Running '{}' preprocessor", self.name());
        Ok(())
    }
}

fn do_it(args: Cli) -> ::std::result::Result<(), Error> {
    let mut book = MDBook::load(args.book)?;
    book.with_preprecessor(ProcessLinks);
    Ok(())
}

main!(|args: Cli, log_level: verbosity| {
    do_it(args).map_err(|e|format_err!("{}", e))?
});
