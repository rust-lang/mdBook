use crate::{get_book_dir, open};
use clap::{arg, App, Arg, ArgMatches};
use mdbook::errors::Result;
use mdbook::MDBook;

// Create clap subcommand arguments
pub fn make_subcommand<'help>() -> App<'help> {
    App::new("build")
        .about("Builds a book from its markdown files")
        .arg(
            Arg::new("dest-dir")
                .short('d')
                .long("dest-dir")
                .value_name("dest-dir")
                .help(
                    "Output directory for the book{n}\
                    Relative paths are interpreted relative to the book's root directory.{n}\
                    If omitted, mdBook uses build.build-dir from book.toml or defaults to `./book`.",
                ),
        )
        .arg(arg!([dir]
            "Root directory for the book{n}\
            (Defaults to the Current Directory when omitted)"
        ))
        .arg(arg!(-o --open "Opens the compiled book in a web browser"))
}

// Build command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(&book_dir)?;

    if let Some(dest_dir) = args.value_of("dest-dir") {
        book.config.build.build_dir = dest_dir.into();
    }

    book.build()?;

    if args.is_present("open") {
        // FIXME: What's the right behaviour if we don't use the HTML renderer?
        open(book.build_dir_for("html").join("index.html"));
    }

    Ok(())
}
