//! High-level library for running mdBook.
//!
//! This is the high-level library for running
//! [mdBook](https://rust-lang.github.io/mdBook/). There are several
//! reasons for using the programmatic API (over the CLI):
//!
//! - Integrate mdBook in a current project.
//! - Extend the capabilities of mdBook.
//! - Do some processing or test before building your book.
//! - Accessing the public API to help create a new Renderer.
//!
//! ## Additional crates
//!
//! In addition to `mdbook-driver`, there are several other crates available
//! for using and extending mdBook:
//!
//! - [`mdbook_preprocessor`]: Provides support for implementing preprocessors.
//! - [`mdbook_renderer`]: Provides support for implementing renderers.
//! - [`mdbook_markdown`]: The Markdown renderer.
//! - [`mdbook_summary`]: The `SUMMARY.md` parser.
//! - [`mdbook_html`]: The HTML renderer.
//! - [`mdbook_core`]: An internal library that is used by the other crates
//!   for shared types. Types from this crate are rexported from the other
//!   crates as appropriate.
//!
//! ## Examples
//!
//! If creating a new book from scratch, you'll want to get a [`init::BookBuilder`] via
//! the [`MDBook::init()`] method.
//!
//! ```rust,no_run
//! use mdbook_driver::MDBook;
//! use mdbook_driver::config::Config;
//!
//! let root_dir = "/path/to/book/root";
//!
//! // create a default config and change a couple things
//! let mut cfg = Config::default();
//! cfg.book.title = Some("My Book".to_string());
//! cfg.book.authors.push("Michael-F-Bryan".to_string());
//!
//! MDBook::init(root_dir)
//!     .create_gitignore(true)
//!     .with_config(cfg)
//!     .build()
//!     .expect("Book generation failed");
//! ```
//!
//! You can also load an existing book and build it.
//!
//! ```rust,no_run
//! use mdbook_driver::MDBook;
//!
//! let root_dir = "/path/to/book/root";
//!
//! let mut md = MDBook::load(root_dir)
//!     .expect("Unable to load the book");
//! md.build().expect("Building failed");
//! ```

pub mod builtin_preprocessors;
pub mod builtin_renderers;
pub mod init;
mod load;
mod mdbook;

use anyhow::{Context, Result, bail};
pub use mdbook::MDBook;
pub use mdbook_core::{book, config, errors};
use shlex::Shlex;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{error, warn};

/// Creates a [`Command`] for command renderers and preprocessors.
fn compose_command(cmd: &str, root: &Path) -> Result<Command> {
    let mut words = Shlex::new(cmd);
    let exe = match words.next() {
        Some(e) => PathBuf::from(e),
        None => bail!("Command string was empty"),
    };

    let exe = if exe.components().count() == 1 {
        // Search PATH for the executable.
        exe
    } else {
        // Relative path is relative to book root.
        root.join(&exe)
    };

    let mut cmd = Command::new(exe);

    for arg in words {
        cmd.arg(arg);
    }

    Ok(cmd)
}

/// Handles a failure for a preprocessor or renderer.
fn handle_command_error(
    error: std::io::Error,
    optional: bool,
    key: &str,
    what: &str,
    name: &str,
    cmd: &str,
) -> Result<()> {
    if let std::io::ErrorKind::NotFound = error.kind() {
        if optional {
            warn!(
                "The command `{cmd}` for {what} `{name}` was not found, \
                 but is marked as optional.",
            );
            return Ok(());
        } else {
            error!(
                "The command `{cmd}` wasn't found, is the `{name}` {what} installed? \
                If you want to ignore this error when the `{name}` {what} is not installed, \
                set `optional = true` in the `[{key}.{name}]` section of the book.toml configuration file.",
            );
        }
    }
    Err(error).with_context(|| format!("Unable to run the {what} `{name}`"))?
}
