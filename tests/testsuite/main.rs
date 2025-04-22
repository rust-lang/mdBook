//! Main testsuite for exercising all functionality of mdBook.
//!
//! See README.md for documentation.

mod book_test;

mod prelude {
    pub use crate::book_test::BookTest;
    pub use snapbox::str;
}
