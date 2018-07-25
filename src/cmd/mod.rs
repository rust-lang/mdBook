//! Subcommand modules for the `mdbook` binary.

pub mod build;
pub mod clean;
pub mod init;
#[cfg(feature = "serve")]
pub mod serve;
pub mod test;
#[cfg(feature = "watch")]
pub mod watch;
