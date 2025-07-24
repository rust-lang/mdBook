//! General build tests.
//!
//! More specific tests should usually go into a module based on the feature.
//! This module should just have general build tests, or misc small things.

use crate::prelude::*;

// Simple smoke test that building works.
#[test]
fn basic_build() {
    BookTest::from_dir("build/basic_build").run("build", |cmd| {
        cmd.expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

"#]]);
    });
}

// Ensure building fails if `create-missing` is false and one of the files does
// not exist.
#[test]
fn failure_on_missing_file() {
    BookTest::from_dir("build/missing_file").run("build", |cmd| {
        cmd.expect_failure().expect_stderr(str![[r#"
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: Chapter file not found, ./chapter_1.md
[TIMESTAMP] [ERROR] (mdbook_core::utils): [TAB]Caused By: [NOT_FOUND]

"#]]);
    });
}

// Ensure a missing file is created if `create-missing` is true.
#[test]
fn create_missing() {
    let test = BookTest::from_dir("build/create_missing");
    assert!(test.dir.join("src/SUMMARY.md").exists());
    assert!(!test.dir.join("src/chapter_1.md").exists());
    test.load_book();
    assert!(test.dir.join("src/chapter_1.md").exists());
}

// Checks that it fails if the summary has a reserved filename.
#[test]
fn no_reserved_filename() {
    BookTest::from_dir("build/no_reserved_filename").run("build", |cmd| {
        cmd.expect_failure().expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: Rendering failed
[TIMESTAMP] [ERROR] (mdbook_core::utils): [TAB]Caused By: print.md is reserved for internal use

"#]]);
    });
}

// Build without book.toml should be OK.
#[test]
fn book_toml_isnt_required() {
    let mut test = BookTest::init(|_| {});
    std::fs::remove_file(test.dir.join("book.toml")).unwrap();
    test.build();
    test.check_main_file(
        "book/chapter_1.html",
        str![[r##"<h1 id="chapter-1"><a class="header" href="#chapter-1">Chapter 1</a></h1>"##]],
    );
}
