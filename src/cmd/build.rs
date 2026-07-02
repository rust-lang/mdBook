use super::command_prelude::*;
use crate::{get_book_dir, open};
use anyhow::Result;
use mdbook_driver::MDBook;
use tracing::error;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("build")
        .about("Builds a book from its markdown files")
        .arg_dest_dir()
        .arg_root_dir()
        .arg_open()
        .arg_copy_exclude_extensions()
}

// Build command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(book_dir)?;

    set_dest_dir(args, &mut book);

    // Apply CLI copy-exclude-extensions to config
    if let Some(exts) = args.get_one::<String>("copy-exclude-extensions") {
        let additional_exts: Vec<String> = exts
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if let Ok(Some(mut html_config)) = book.config.get::<mdbook_core::config::HtmlConfig>("output.html") {
            html_config.copy_exclude_extensions.extend(additional_exts);
            book.config.set("output.html", html_config)?;
        }
    }

    book.build()?;

    if args.get_flag("open") {
        // FIXME: What's the right behaviour if we don't use the HTML renderer?
        let path = book.build_dir_for("html").join("index.html");
        if !path.exists() {
            error!("No chapter available to open");
            std::process::exit(1)
        }
        open(path);
    }

    Ok(())
}
