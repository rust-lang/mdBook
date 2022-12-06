use crate::utils::ignore::remove_ignored_files;
use crate::{get_book_dir, open};
use clap::{arg, App, Arg, ArgMatches};
use mdbook::errors::Result;
use mdbook::utils;
use mdbook::MDBook;
use notify::Watcher;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;

// Create clap subcommand arguments
pub fn make_subcommand<'help>() -> App<'help> {
    App::new("watch")
        .about("Watches a book's files and rebuilds it on changes")
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

// Watch command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(&book_dir)?;

    let update_config = |book: &mut MDBook| {
        if let Some(dest_dir) = args.value_of("dest-dir") {
            book.config.build.build_dir = dest_dir.into();
        }
    };
    update_config(&mut book);

    if args.is_present("open") {
        book.build()?;
        let path = book.build_dir_for("html").join("index.html");
        if !path.exists() {
            error!("No chapter available to open");
            std::process::exit(1)
        }
        open(path);
    }

    trigger_on_change(&book, |paths, book_dir| {
        info!("Files changed: {:?}\nBuilding book...\n", paths);
        let result = MDBook::load(&book_dir).and_then(|mut b| {
            update_config(&mut b);
            b.build()
        });

        if let Err(e) = result {
            error!("Unable to build the book");
            utils::log_backtrace(&e);
        }
    });

    Ok(())
}

/// Calls the closure when a book source file is changed, blocking indefinitely.
pub fn trigger_on_change<F>(book: &MDBook, closure: F)
where
    F: Fn(Vec<PathBuf>, &Path),
{
    use notify::DebouncedEvent::*;
    use notify::RecursiveMode::*;

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
        Ok(w) => w,
        Err(e) => {
            error!("Error while trying to watch the files:\n\n\t{:?}", e);
            std::process::exit(1)
        }
    };

    // Add the source directory to the watcher
    if let Err(e) = watcher.watch(book.source_dir(), Recursive) {
        error!("Error while watching {:?}:\n    {:?}", book.source_dir(), e);
        std::process::exit(1);
    };

    let _ = watcher.watch(book.theme_dir(), Recursive);

    // Add the book.toml file to the watcher if it exists
    let _ = watcher.watch(book.root.join("book.toml"), NonRecursive);

    for dir in &book.config.build.extra_watch_dirs {
        let path = dir.canonicalize().unwrap();
        if let Err(e) = watcher.watch(&path, Recursive) {
            error!(
                "Error while watching extra directory {:?}:\n    {:?}",
                path, e
            );
            std::process::exit(1);
        }
    }

    info!("Listening for changes...");

    loop {
        let first_event = rx.recv().unwrap();
        sleep(Duration::from_millis(50));
        let other_events = rx.try_iter();

        let all_events = std::iter::once(first_event).chain(other_events);

        let paths = all_events
            .filter_map(|event| {
                debug!("Received filesystem event: {:?}", event);

                match event {
                    Create(path) | Write(path) | Remove(path) | Rename(_, path) => Some(path),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        // If we are watching files outside the current repository (via extra-watch-dirs), then they are definitionally
        // ignored by gitignore. So we handle this case by including such files into the watched paths list.
        let any_external_paths = paths.iter().filter(|p| !p.starts_with(&book.root)).cloned();
        let mut paths = remove_ignored_files(&book.root, &paths[..]);
        paths.extend(any_external_paths);

        if !paths.is_empty() {
            closure(paths, &book.root);
        }
    }
}
