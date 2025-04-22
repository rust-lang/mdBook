//! Tests for `mdbook init`.

use crate::prelude::*;
use mdbook::MDBook;

// Tests "init" with no args.
#[test]
fn basic_init() {
    let mut test = BookTest::empty();
    test.run("init", |cmd| {
        cmd.expect_stdout(str![[r#"

Do you want a .gitignore to be created? (y/n)
What title would you like to give the book? 

All done, no errors...

"#]])
            .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook::book::init): Creating a new book with stub content

"#]]);
    })
    .check_file(
        "book.toml",
        str![[r#"
[book]
authors = []
language = "en"
src = "src"

"#]],
    )
    .check_file(
        "src/SUMMARY.md",
        str![[r#"
# Summary

- [Chapter 1](./chapter_1.md)

"#]],
    )
    .check_file(
        "src/chapter_1.md",
        str![[r#"
# Chapter 1

"#]],
    )
    .check_main_file(
        "book/chapter_1.html",
        str![[r##"<h1 id="chapter-1"><a class="header" href="#chapter-1">Chapter 1</a></h1>"##]],
    );
    assert!(!test.dir.join(".gitignore").exists());
    assert!(test.dir.join("book").exists());
}

// Test init via API. This does a little less than the CLI does.
#[test]
fn init_api() {
    let mut test = BookTest::empty();
    MDBook::init(&test.dir).build().unwrap();
    test.check_file_list(
        ".",
        str![[r#"
book
book.toml
src
src/SUMMARY.md
src/chapter_1.md
"#]],
    );
}

// Run `mdbook init` with `--force` to skip the confirmation prompts
#[test]
fn init_force() {
    let mut test = BookTest::empty();
    test.run("init --force", |cmd| {
        cmd.expect_stdout(str![[r#"

All done, no errors...

"#]])
            .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook::book::init): Creating a new book with stub content

"#]]);
    })
    .check_file(
        "book.toml",
        str![[r#"
[book]
authors = []
language = "en"
src = "src"

"#]],
    );
    assert!(!test.dir.join(".gitignore").exists());
}
