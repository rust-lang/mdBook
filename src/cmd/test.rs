use clap::{App, ArgMatches, SubCommand};
use get_book_dir;
use mdbook::errors::Result;
use mdbook::MDBook;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("test")
        .about("Test that code samples compile")
        .arg_from_usage("-L, --library-path [DIR]... 'directories to add to crate search path'")
        .arg_from_usage(
            "[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'",
        )
}

// test command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let library_paths: Vec<&str> = args.values_of("library-path")
        .map(|v| v.collect())
        .unwrap_or_default();
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(&book_dir)?;

    book.test(library_paths)?;

    Ok(())
}
