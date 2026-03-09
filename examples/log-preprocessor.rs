//! A basic example of a preprocessor that does nothing besides using mdBook's logger.

use clap::{Arg, Command};
use mdbook_preprocessor::book::Book;
use mdbook_preprocessor::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use std::{io, process};

use crate::log_lib::Log;

fn make_app() -> Command {
    Command::new("log-preprocessor")
        .about("A mdbook preprocessor which logs a couple messages")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() -> Result<()> {
    let matches = make_app().get_matches();

    if let Some(_) = matches.subcommand_matches("supports") {
        process::exit(0);
    } else {
        let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;
        let processed_book = Log.run(&ctx, book)?;
        serde_json::to_writer(io::stdout(), &processed_book)?;
        Ok(())
    }
}

#[allow(unreachable_pub, reason = "wouldn't be a problem in a proper lib.rs")]
mod log_lib {
    use mdbook_core::utils::init_logger;
    use tracing::{info, warn};

    use super::*;

    #[allow(dead_code)]
    pub struct Log;

    impl Preprocessor for Log {
        fn name(&self) -> &str {
            "log-preprocessor"
        }

        fn run(&self, _ctx: &PreprocessorContext, book: Book) -> Result<Book> {
            init_logger(); // mandatory to properly use mdBook's logger
            info!("It works");
            warn!("It works");
            Ok(book)
        }
    }
}
