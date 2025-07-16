use super::command_prelude::*;
use crate::{get_backends, get_book_dir, open};
use mdbook::MDBook;
use mdbook::{book::ActiveBackends, errors::Result};
use std::path::{Path, PathBuf};

mod native;
mod poller;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("watch")
        .about("Watches a book's files and rebuilds it on changes")
        .arg_dest_dir()
        .arg_root_dir()
        .arg_open()
        .arg_backends()
        .arg_watcher()
}

pub enum WatcherKind {
    Poll,
    Native,
}

impl WatcherKind {
    pub fn from_str(s: &str) -> WatcherKind {
        match s {
            "poll" => WatcherKind::Poll,
            "native" => WatcherKind::Native,
            _ => panic!("unsupported watcher {s}"),
        }
    }
}

// Watch command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(&book_dir)?;

    let active_backends = get_backends(args);

    let update_config = |book: &mut MDBook| {
        if let Some(dest_dir) = args.get_one::<PathBuf>("dest-dir") {
            book.config.build.build_dir = dest_dir.into();
        }
    };
    update_config(&mut book);

    if args.get_flag("open") {
        book.render(&active_backends)?;
        let path = book.build_dir_for("html").join("index.html");
        if !path.exists() {
            error!("No chapter available to open");
            std::process::exit(1)
        }
        open(path);
    }

    let watcher = WatcherKind::from_str(args.get_one::<String>("watcher").unwrap());
    rebuild_on_change(watcher, &book_dir, &update_config, &active_backends, &|| {});

    Ok(())
}

pub fn rebuild_on_change(
    kind: WatcherKind,
    book_dir: &Path,
    update_config: &dyn Fn(&mut MDBook),
    backends: &ActiveBackends,
    post_build: &dyn Fn(),
) {
    match kind {
        WatcherKind::Poll => {
            self::poller::rebuild_on_change(book_dir, update_config, backends, post_build)
        }
        WatcherKind::Native => {
            self::native::rebuild_on_change(book_dir, update_config, backends, post_build)
        }
    }
}

fn find_gitignore(book_root: &Path) -> Option<PathBuf> {
    book_root
        .ancestors()
        .map(|p| p.join(".gitignore"))
        .find(|p| p.exists())
}
