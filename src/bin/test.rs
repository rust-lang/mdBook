use clap::{ArgMatches, SubCommand, App};
use mdbook::MDBook;
use mdbook::errors::Result;
use get_book_dir;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("test")
        .about("Test that code samples compile")
        .arg_from_usage("-L, --library-path [DIR]... 'directory to add to crate search path'")
}

// test command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let library_paths: Vec<&str> = args.values_of("library-path").map(|v| v.collect()).unwrap_or_default();
    let book_dir = get_book_dir(args);
    let mut book = MDBook::new(&book_dir).read_config()?;

    book.test(library_paths)?;

    Ok(())
}
