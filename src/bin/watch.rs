extern crate notify;

use std::path::{Path, PathBuf};
use self::notify::Watcher;
use std::time::Duration;
use std::sync::mpsc::channel;
use clap::{App, ArgMatches, SubCommand};
use mdbook::MDBook;
use mdbook::errors::Result;
use {get_book_dir, open};

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("watch")
        .about("Watch the files for changes")
        .arg_from_usage("-o, --open 'Open the compiled book in a web browser'")
        .arg_from_usage(
            "-d, --dest-dir=[dest-dir] 'The output directory for \
             your book{n}(Defaults to ./book when omitted)'",
        )
        .arg_from_usage(
            "[dir] 'A directory for your book{n}(Defaults to \
             Current Directory when omitted)'",
        )
}

// Watch command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::new(&book_dir).read_config()?;

    if let Some(dest_dir) = args.value_of("dest-dir") {
        book.config.build.build_dir = PathBuf::from(dest_dir);
    }

    if args.is_present("open") {
        book.build()?;
        open(book.get_destination().join("index.html"));
    }

    trigger_on_change(&mut book, |path, book| {
        println!("File changed: {:?}\nBuilding book...\n", path);
        if let Err(e) = book.build() {
            println!("Error while building: {:?}", e);
        }
        println!("");
    });

    Ok(())
}

// Calls the closure when a book source file is changed. This is blocking!
pub fn trigger_on_change<F>(book: &mut MDBook, closure: F) -> ()
where
    F: Fn(&Path, &mut MDBook) -> (),
{
    use self::notify::RecursiveMode::*;
    use self::notify::DebouncedEvent::*;

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
        Ok(w) => w,
        Err(e) => {
            println!("Error while trying to watch the files:\n\n\t{:?}", e);
            ::std::process::exit(0)
        }
    };

    // Add the source directory to the watcher
    if let Err(e) = watcher.watch(book.get_source(), Recursive) {
        println!("Error while watching {:?}:\n    {:?}", book.get_source(), e);
        ::std::process::exit(0);
    };

    // Add the theme directory to the watcher
    watcher.watch(book.theme_dir(), Recursive)
           .unwrap_or_default();

    // Add the book.{json,toml} file to the watcher if it exists, because it's not
    // located in the source directory
    if watcher.watch(book.root.join("book.json"), NonRecursive)
              .is_err()
    {
        // do nothing if book.json is not found
    }
    if watcher.watch(book.root.join("book.toml"), NonRecursive)
              .is_err()
    {
        // do nothing if book.toml is not found
    }

    println!("\nListening for changes...\n");

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    Create(path) | Write(path) | Remove(path) | Rename(_, path) => {
                        closure(&path, book);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                println!("An error occured: {:?}", e);
            }
        }
    }
}
