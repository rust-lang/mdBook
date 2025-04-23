//! Main testsuite for exercising all functionality of mdBook.
//!
//! See README.md for documentation.

mod book_test;
mod build;
mod cli;
mod includes;
mod index;
mod init;
mod markdown;
mod playground;
mod preprocessor;
mod print;
mod redirects;
mod renderer;
mod rendering;
#[cfg(feature = "search")]
mod search;
mod test;
mod theme;
mod toc;

mod prelude {
    pub use crate::book_test::BookTest;
    pub use snapbox::str;
}
