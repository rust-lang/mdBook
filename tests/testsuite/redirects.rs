//! Tests for the HTML redirect feature.

use crate::prelude::*;
use snapbox::file;

// Basic check of redirects.
#[test]
fn redirects_are_emitted_correctly() {
    BookTest::from_dir("redirects/redirects_are_emitted_correctly")
        .check_file(
            "book/overview.html",
            file!["redirects/redirects_are_emitted_correctly/expected/overview.html"],
        )
        .check_file(
            "book/nested/page.html",
            file!["redirects/redirects_are_emitted_correctly/expected/nested/page.html"],
        );
}
