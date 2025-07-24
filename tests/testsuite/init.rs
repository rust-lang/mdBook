//! Tests for `mdbook init`.

use crate::prelude::*;
use mdbook_core::config::Config;
use mdbook_driver::MDBook;
use std::path::PathBuf;

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
[TIMESTAMP] [INFO] (mdbook_driver::init): Creating a new book with stub content

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
[TIMESTAMP] [INFO] (mdbook_driver::init): Creating a new book with stub content

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

// Run `mdbook init` with `--title` without git config.
//
// Regression test for https://github.com/rust-lang/mdBook/issues/2485
#[test]
fn no_git_config_with_title() {
    let mut test = BookTest::empty();
    test.run("init", |cmd| {
        cmd.expect_stdout(str![[r#"

Do you want a .gitignore to be created? (y/n)

All done, no errors...

"#]])
            .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::init): Creating a new book with stub content

"#]])
            .args(&["--title", "Example title"]);
    })
    .check_file(
        "book.toml",
        str![[r#"
[book]
authors = []
language = "en"
src = "src"
title = "Example title"

"#]],
    );
    assert!(!test.dir.join(".gitignore").exists());
}

// Run `mdbook init` in a directory containing a SUMMARY.md should create the
// files listed in the summary.
#[test]
fn init_from_summary() {
    BookTest::from_dir("init/init_from_summary")
        .run("init", |_| {})
        .check_file(
            "src/intro.md",
            str![[r#"
# intro

"#]],
        )
        .check_file(
            "src/first.md",
            str![[r#"
# First chapter

"#]],
        )
        .check_file(
            "src/outro.md",
            str![[r#"
# outro

"#]],
        );
}

// Set some custom arguments for where to place the source and destination
// files, then call `mdbook init`.
#[test]
fn init_with_custom_book_and_src_locations() {
    let mut test = BookTest::empty();
    let mut cfg = Config::default();
    cfg.book.src = PathBuf::from("in");
    cfg.build.build_dir = PathBuf::from("out");
    MDBook::init(&test.dir).with_config(cfg).build().unwrap();
    test.check_file(
        "book.toml",
        str![[r#"
[book]
authors = []
language = "en"
src = "in"

[build]
build-dir = "out"
create-missing = true
extra-watch-dirs = []
use-default-preprocessors = true

"#]],
    )
    .check_file(
        "in/SUMMARY.md",
        str![[r#"
# Summary

- [Chapter 1](./chapter_1.md)

"#]],
    )
    .check_file(
        "in/chapter_1.md",
        str![[r#"
# Chapter 1

"#]],
    );
    assert!(test.dir.join("out").exists());
}

// Copies the theme into the initialized directory.
#[test]
fn copy_theme() {
    BookTest::empty()
        .run("init --theme", |_| {})
        .check_file_list(
            ".",
            str![[r#"
book
book.toml
src
src/SUMMARY.md
src/chapter_1.md
theme
theme/book.js
theme/css
theme/css/chrome.css
theme/css/general.css
theme/css/print.css
theme/css/variables.css
theme/favicon.png
theme/favicon.svg
theme/fonts
theme/fonts/OPEN-SANS-LICENSE.txt
theme/fonts/SOURCE-CODE-PRO-LICENSE.txt
theme/fonts/fonts.css
theme/fonts/open-sans-v17-all-charsets-300.woff2
theme/fonts/open-sans-v17-all-charsets-300italic.woff2
theme/fonts/open-sans-v17-all-charsets-600.woff2
theme/fonts/open-sans-v17-all-charsets-600italic.woff2
theme/fonts/open-sans-v17-all-charsets-700.woff2
theme/fonts/open-sans-v17-all-charsets-700italic.woff2
theme/fonts/open-sans-v17-all-charsets-800.woff2
theme/fonts/open-sans-v17-all-charsets-800italic.woff2
theme/fonts/open-sans-v17-all-charsets-italic.woff2
theme/fonts/open-sans-v17-all-charsets-regular.woff2
theme/fonts/source-code-pro-v11-all-charsets-500.woff2
theme/highlight.css
theme/highlight.js
theme/index.hbs
"#]],
        );
}
