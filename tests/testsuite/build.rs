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
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `[ROOT]/book`

"#]]);
    });
}

// Ensure building fails if `create-missing` is false and one of the files does
// not exist.
#[test]
fn failure_on_missing_file() {
    BookTest::from_dir("build/missing_file").run("build", |cmd| {
        cmd.expect_failure().expect_stderr(str![[r#"
ERROR Chapter file not found, ./chapter_1.md
[TAB]Caused by: [NOT_FOUND]

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
 INFO Book building has started
 INFO Running the html backend
ERROR Rendering failed
[TAB]Caused by: print.md is reserved for internal use

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

// Dest dir relative path behavior.
#[test]
fn dest_dir_relative_path() {
    let mut test = BookTest::from_dir("build/basic_build");
    let current_dir = test.dir.join("work");
    std::fs::create_dir_all(&current_dir).unwrap();
    test.run("build", |cmd| {
        cmd.args(&["--dest-dir", "foo", ".."])
            .current_dir(&current_dir)
            .expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `[ROOT]/work/foo`

"#]]);
    });
    assert!(current_dir.join("foo/index.html").exists());
}
