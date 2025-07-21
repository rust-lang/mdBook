//! The base support library for mdbook, intended for internal use only.

/// The current version of `mdbook`.
///
/// This is provided as a way for custom preprocessors and renderers to do
/// compatibility checks.
pub const MDBOOK_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod book;
pub mod config;
pub mod utils;

/// The error types used in mdbook.
pub mod errors {
    pub use anyhow::{Error, Result};
}
