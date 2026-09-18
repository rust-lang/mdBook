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
//! ## Cargo features
//!
//! The following cargo features are available:
//!
//! - `search`: Enables the search index in the HTML renderer.
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

/// Returns whether a command is definitely unavailable.
///
/// Checking before spawning avoids relying on `execvp`'s error reporting. If
/// `PATH` contains an inaccessible directory, a missing executable can be
/// reported as `PermissionDenied` instead of `NotFound`, which prevents
/// optional extensions from being skipped. The same preflight is used on all
/// platforms, while accounting for each platform's command-search rules.
fn command_is_missing(command: &Command) -> bool {
    let program = Path::new(command.get_program());
    let current_dir = command.get_current_dir();
    let resolve = |path: &Path| match (path.is_absolute(), current_dir) {
        (true, _) | (false, None) => path.to_path_buf(),
        (false, Some(current_dir)) => current_dir.join(path),
    };

    if program.components().count() > 1 {
        return command_candidates(resolve(program))
            .into_iter()
            .all(|path| command_path_is_missing(&path));
    }

    let Some(directories) = command_search_directories() else {
        // Without PATH on Unix, execvp uses an implementation-defined default
        // search path, so let spawn report the result instead of guessing.
        return false;
    };

    let mut checked_candidate = false;
    for directory in directories {
        for candidate in command_candidates(resolve(&directory).join(program)) {
            checked_candidate = true;
            if !command_path_is_missing_or_inaccessible(&candidate) {
                return false;
            }
        }
    }

    checked_candidate
}

/// Returns the places `Command` searches for a bare program name.
///
/// `compose_command` does not override a child process's environment, so the
/// current process's `PATH` is also the child's `PATH` here.
#[cfg(not(windows))]
fn command_search_directories() -> Option<Vec<PathBuf>> {
    let path = std::env::var_os("PATH")?;
    let directories: Vec<_> = std::env::split_paths(&path).collect();
    (!directories.is_empty()).then_some(directories)
}

/// Mirrors the documented Windows search order used by [`Command`].
#[cfg(windows)]
fn command_search_directories() -> Option<Vec<PathBuf>> {
    let executable = std::env::current_exe().ok()?;
    let executable_directory = executable.parent()?.to_path_buf();
    let windows_directory = std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("WINDIR"))
        .map(PathBuf::from)?;

    let mut directories = vec![
        executable_directory,
        windows_directory.join("System32"),
        windows_directory,
    ];
    if let Some(path) = std::env::var_os("PATH") {
        directories.extend(
            std::env::split_paths(&path).filter(|directory| !directory.as_os_str().is_empty()),
        );
    }
    Some(directories)
}

/// Returns the filenames [`Command`] will try for a command path.
#[cfg(not(windows))]
fn command_candidates(path: PathBuf) -> Vec<PathBuf> {
    vec![path]
}

/// On Windows, `Command` permits omitting an `.exe` extension.
#[cfg(windows)]
fn command_candidates(path: PathBuf) -> Vec<PathBuf> {
    if path.extension().is_some() {
        vec![path]
    } else {
        vec![path.with_extension("exe")]
    }
}

/// A direct command path is missing only when it cannot be found.
fn command_path_is_missing(path: &Path) -> bool {
    matches!(
        std::fs::metadata(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound
    )
}

/// A `PATH` entry that cannot be searched is equivalent to no usable command
/// at that entry. Other filesystem errors still go through `spawn` unchanged.
fn command_path_is_missing_or_inaccessible(path: &Path) -> bool {
    matches!(
        std::fs::metadata(path),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
            )
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn non_executable_direct_command_is_not_considered_missing() {
        use std::os::unix::fs::PermissionsExt;

        let temporary_directory = tempfile::tempdir().unwrap();
        let command_path = temporary_directory.path().join("command");
        std::fs::write(&command_path, b"").unwrap();
        std::fs::set_permissions(&command_path, std::fs::Permissions::from_mode(0o600)).unwrap();

        let command = Command::new(command_path);
        assert!(!command_is_missing(&command));
    }

    #[cfg(windows)]
    #[test]
    fn extensionless_windows_command_is_not_considered_missing_when_exe_exists() {
        let temporary_directory = tempfile::tempdir().unwrap();
        let executable = temporary_directory.path().join("renderer.exe");
        std::fs::write(&executable, b"").unwrap();

        let command = Command::new(temporary_directory.path().join("renderer"));
        assert!(!command_is_missing(&command));
    }
}
