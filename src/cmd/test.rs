use crate::get_book_dir;
use clap::{arg, App, Arg, ArgMatches};
use mdbook::errors::Result;
use mdbook::MDBook;

const EXTERN_HELP: &str = "Specify the name and location of an external crate. This argument 
gets passed down to rustdoc. To see the set of --extern and --library-path
arguments needed to use a given crate, create a simple example project that
uses said crate and run `cargo doc -v.`

Example: --extern my_crate=/path/to/crate.rlib";

// Create clap subcommand arguments
pub fn make_subcommand<'help>() -> App<'help> {
    App::new("test")
        .about("Tests that a book's Rust code samples compile")
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
        .arg(Arg::new("library-path")
            .short('L')
            .long("library-path")
            .value_name("dir")
            .takes_value(true)
            .use_delimiter(true)
            .require_delimiter(true)
            .multiple_values(true)
            .multiple_occurrences(true)
            .forbid_empty_values(true)
            .help("A comma-separated list of directories to add to {n}the crate search path when building tests"))
        .arg(Arg::with_name("extern")
            .long("extern")
            .value_name("file")
            .takes_value(true)
            .require_delimiter(false)
            .multiple(true)
            .empty_values(false)
            .help(EXTERN_HELP))
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .takes_value(false)
            .multiple(false)
            .help("Enables verbose logging with the test command."))
}

// test command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let library_paths: Vec<&str> = args
        .values_of("library-path")
        .map(std::iter::Iterator::collect)
        .unwrap_or_default();
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(&book_dir)?;
    let externs: Vec<&str> = args
        .values_of("extern")
        .map(std::iter::Iterator::collect)
        .unwrap_or_default();

    if let Some(dest_dir) = args.value_of("dest-dir") {
        book.config.build.build_dir = dest_dir.into();
    }

    book.test(library_paths, externs, args.is_present("verbose"))?;

    Ok(())
}
