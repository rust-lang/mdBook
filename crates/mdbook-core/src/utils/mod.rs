//! Various helpers and utilities.

use anyhow::Error;
use std::fmt::Write;
use tracing::error;

pub mod fs;
mod html;
mod string;
mod toml_ext;

pub(crate) use self::toml_ext::TomlExt;

pub use self::html::{escape_html, escape_html_attribute};
pub use self::string::{
    take_anchored_lines, take_lines, take_rustdoc_include_anchored_lines,
    take_rustdoc_include_lines,
};

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
