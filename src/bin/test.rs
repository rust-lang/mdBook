use clap::{ArgMatches, SubCommand, App};
use mdbook::MDBook;
use mdbook::errors::Result;
use get_book_dir;

// test command implementation
pub fn test(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::new(&book_dir).read_config()?;

    book.test()?;

    Ok(())
}

pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("test").about("Test that code samples compile")
}
