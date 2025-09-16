//! Tests for print page.

use crate::prelude::*;
use snapbox::file;

// Tests relative links from the print page.
#[test]
fn relative_links() {
    BookTest::from_dir("print/relative_links").check_main_file(
        "book/print.html",
        file!("print/relative_links/expected/print.html"),
    );
}

// Test for duplicate IDs, and links to those duplicates.
#[test]
fn duplicate_ids() {
    BookTest::from_dir("print/duplicate_ids").check_main_file(
        "book/print.html",
        file!("print/duplicate_ids/expected/print.html"),
    );
}

// Test for synthesized link to a chapter that does not have an h1.
#[test]
fn chapter_no_h1() {
    BookTest::from_dir("print/chapter_no_h1").check_main_file(
        "book/print.html",
        file!("print/chapter_no_h1/expected/print.html"),
    );
}

// Checks that print.html is noindex.
#[test]
fn noindex() {
    let robots = r#"<meta name="robots" content="noindex">"#;
    BookTest::from_dir("print/noindex")
        .check_file_contains("book/print.html", robots)
        .check_file_doesnt_contain("book/index.html", robots);
}
