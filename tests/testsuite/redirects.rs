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

// Invalid redirect with only fragments.
#[test]
fn redirect_removed_with_fragments_only() {
    BookTest::from_dir("redirects/redirect_removed_with_fragments_only").run("build", |cmd| {
        cmd.expect_failure().expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
ERROR Rendering failed
[TAB]Caused by: Unable to emit redirects
[TAB]Caused by: redirect entry for `old-file.html` only has source paths with `#` fragments
There must be an entry without the `#` fragment to determine the default destination.

"#]]);
    });
}

// Invalid redirect for an existing page.
#[test]
fn redirect_existing_page() {
    BookTest::from_dir("redirects/redirect_existing_page").run("build", |cmd| {
        cmd.expect_failure().expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
ERROR Rendering failed
[TAB]Caused by: redirect found for existing chapter at `/chapter_1.html`
Either delete the redirect or remove the chapter.

"#]]);
    });
}
