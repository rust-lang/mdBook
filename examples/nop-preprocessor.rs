extern crate mdbook;
extern crate serde_json;
#[macro_use]
extern crate clap;

use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::preprocess::CmdPreprocessor;
use mdbook::book::Book;
use std::process;
use std::io;

fn main() {
    let matches = app().get_matches();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(sub_args);
    } else {
        handle_preprocessing();
    }
}

fn handle_preprocessing() {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())
        .expect("Couldn't parse the input");

    // You can inspect the calling mdbook's version to check for compatibility
    if ctx.mdbook_version != mdbook::MDBOOK_VERSION  {
        panic!("The version check failed!");
    }

    // In testing we want to tell the preprocessor to blow up by setting a
    // particular config value
    if let Some(table) = ctx.config.get_preprocessor("nop-preprocessor") {
        let should_blow_up = table.get("blow-up").is_some();

        if should_blow_up {
            panic!("Boom!!!1!");
        }
    }

    let processed_book = do_processing(book);

    serde_json::to_writer(io::stdout(), &processed_book).unwrap();
}

fn do_processing(book: Book) -> Book {
    // We *are* a nop preprocessor after all...
    book
}

fn handle_supports(sub_args: &ArgMatches) {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = renderer_is_supported(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn renderer_is_supported(renderer: &str) -> bool {
    // We support everything except the `not-supported` renderer
    renderer != "not-supported"
}

fn app() -> App<'static, 'static> {
    app_from_crate!().subcommand(
        SubCommand::with_name("supports")
            .arg(Arg::with_name("renderer").required(true))
            .about(
                "Check whether a renderer is supported by this preprocessor",
            ),
    )
}
