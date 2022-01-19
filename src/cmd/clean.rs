use crate::get_book_dir;
use anyhow::Context;
use clap::{arg, App, Arg, ArgMatches};
use mdbook::MDBook;
use std::fs;

// Create clap subcommand arguments
pub fn make_subcommand<'help>() -> App<'help> {
    App::new("clean")
        .about("Deletes a built book")
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
}

// Clean command implementation
pub fn execute(args: &ArgMatches) -> mdbook::errors::Result<()> {
    let book_dir = get_book_dir(args);
    let book = MDBook::load(&book_dir)?;

    let dir_to_remove = match args.value_of("dest-dir") {
        Some(dest_dir) => dest_dir.into(),
        None => book.root.join(&book.config.build.build_dir),
    };

    if dir_to_remove.exists() {
        fs::remove_dir_all(&dir_to_remove)
            .with_context(|| "Unable to remove the build directory")?;
    }

    Ok(())
}
