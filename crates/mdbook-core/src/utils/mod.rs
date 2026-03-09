//! Various helpers and utilities.

use anyhow::Error;
use std::fmt::Write;
use tracing::error;

pub mod fs;
mod html;
mod toml_ext;

/// Initialize the logger with mdBook's default configuration.
///
/// This function sets up a tracing subscriber that:
/// - Uses the `MDBOOK_LOG` environment variable for filtering (or defaults to INFO level)
/// - Silences noisy dependencies (handlebars, html5ever) unless explicitly requested
/// - Writes to stderr
/// - Shows the target only when MDBOOK_LOG is set
///
/// This is useful for preprocessor authors who want consistent logging
/// with mdBook's internal logging.
pub fn init_logger() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_env_var("MDBOOK_LOG")
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    let log_env = std::env::var("MDBOOK_LOG");
    // Silence some particularly noisy dependencies unless the user
    // specifically asks for them.
    let silence_unless_specified = |filter: tracing_subscriber::EnvFilter, target| {
        if !log_env.as_ref().map_or(false, |s| {
            s.split(',').any(|directive| directive.starts_with(target))
        }) {
            filter.add_directive(format!("{target}=warn").parse().unwrap())
        } else {
            filter
        }
    };
    let filter = silence_unless_specified(filter, "handlebars");
    let filter = silence_unless_specified(filter, "html5ever");

    // Don't show the target by default, since it generally isn't useful
    // unless you are overriding the level.
    let with_target = log_env.is_ok();

    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .with_target(with_target)
        .init();
}

pub(crate) use self::toml_ext::TomlExt;

pub use self::html::{escape_html, escape_html_attribute};

/// Defines a `static` with a [`regex::Regex`].
#[macro_export]
macro_rules! static_regex {
    ($name:ident, $regex:literal) => {
        static $name: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new($regex).unwrap());
    };
    ($name:ident, bytes, $regex:literal) => {
        static $name: std::sync::LazyLock<regex::bytes::Regex> =
            std::sync::LazyLock::new(|| regex::bytes::Regex::new($regex).unwrap());
    };
}

/// Prints a "backtrace" of some `Error`.
pub fn log_backtrace(e: &Error) {
    let mut message = format!("{e}");

    for cause in e.chain().skip(1) {
        write!(message, "\n\tCaused by: {cause}").unwrap();
    }

    error!("{message}");
}
