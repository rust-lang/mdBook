extern crate notify;

use std::path::Path;
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
            "[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'",
        )
}

// Watch command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(&book_dir)?;

    if args.is_present("open") {
        book.build()?;
        open(book.get_destination().join("index.html"));
    }

    trigger_on_change(&book, |path, book_dir| {
        println!("File changed: {:?}\nBuilding book...\n", path);
        let result = MDBook::load(&book_dir).and_then(|mut b| b.build());

        if let Err(e) = result {
            println!("Error while building: {}", e);
        }
        println!();
    });

    Ok(())
}

/// Calls the closure when a book source file is changed, blocking indefinitely.
pub fn trigger_on_change<F>(book: &MDBook, closure: F)
where
    F: Fn(&Path, &Path),
{
    use self::notify::RecursiveMode::*;
    use self::notify::DebouncedEvent::*;

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
        Ok(w) => w,
        Err(e) => {
            println!("Error while trying to watch the files:\n\n\t{:?}", e);
            ::std::process::exit(1)
        }
    };

    // Add the source directory to the watcher
    if let Err(e) = watcher.watch(book.source_dir(), Recursive) {
        println!("Error while watching {:?}:\n    {:?}", book.source_dir(), e);
        ::std::process::exit(1);
    };

    let _ = watcher.watch(book.theme_dir(), Recursive);

    // Add the book.toml file to the watcher if it exists
    let _ = watcher.watch(book.root.join("book.toml"), NonRecursive);

    println!("\nListening for changes...\n");

    for event in rx.recv() {
        match event {
            Create(path) | Write(path) | Remove(path) | Rename(_, path) => {
                closure(&path, &book.root);
            }
            _ => {}
        }
    }
}
