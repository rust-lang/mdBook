use std::fs;
use std::path::PathBuf;
use clap::{App, ArgMatches, SubCommand};
use mdbook::MDBook;
use mdbook::errors::*;
use get_book_dir;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("clean")
        .about("Delete built book")
        .arg_from_usage(
            "-d, --dest-dir=[dest-dir] 'The directory of built book{n}(Defaults to ./book when \
             omitted)'",
        )
}

// Clean command implementation
pub fn execute(args: &ArgMatches) -> ::mdbook::errors::Result<()> {
    let book_dir = get_book_dir(args);
    let book = MDBook::load(&book_dir)?;

    let dir_to_remove = match args.value_of("dest-dir") {
        Some(dest_dir) => PathBuf::from(dest_dir),
        None => book.root.join(&book.config.build.build_dir),
    };
    fs::remove_dir_all(&dir_to_remove).chain_err(|| "Unable to remove the build directory")?;

    Ok(())
}
