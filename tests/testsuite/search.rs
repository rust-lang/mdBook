//! Tests for search support.

use crate::prelude::*;
use mdbook_core::book::{BookItem, Chapter};
use snapbox::file;
use std::path::{Path, PathBuf};

fn read_book_index(root: &Path) -> serde_json::Value {
    let index = root.join("book/searchindex.js");
    let index = std::fs::read_to_string(index).unwrap();
    let index =
        index.trim_start_matches("window.search = Object.assign(window.search, JSON.parse('");
    let index = index.trim_end_matches("'));");
    // We need unescape the string as it's supposed to be an escaped JS string.
    serde_json::from_str(&index.replace("\\'", "'").replace("\\\\", "\\")).unwrap()
}

// Some spot checks for the generation of the search index.
#[test]
fn reasonable_search_index() {
    let mut test = BookTest::from_dir("search/reasonable_search_index");
    test.build();
    let index = read_book_index(&test.dir);

    let doc_urls = index["doc_urls"].as_array().unwrap();
    eprintln!("doc_urls={doc_urls:#?}",);
    let get_doc_ref = |url: &str| -> String {
        doc_urls
            .iter()
            .position(|s| s == url)
            .unwrap_or_else(|| panic!("failed to find {url}"))
            .to_string()
    };

    let first_chapter = get_doc_ref("first/index.html#first-chapter");
    let introduction = get_doc_ref("intro.html#introduction");
    let some_section = get_doc_ref("first/index.html#some-section");
    let summary = get_doc_ref("first/includes.html#summary");
    let no_headers = get_doc_ref("first/no-headers.html");
    let duplicate_headers_1 = get_doc_ref("first/duplicate-headers.html#header-text-1");
    let heading_attrs = get_doc_ref("first/heading-attributes.html#both");
    let sneaky = get_doc_ref("intro.html#sneaky");

    let bodyidx = &index["index"]["index"]["body"]["root"];
    let textidx = &bodyidx["t"]["e"]["x"]["t"];
    assert_eq!(textidx["df"], 5);
    assert_eq!(textidx["docs"][&first_chapter]["tf"], 1.0);
    assert_eq!(textidx["docs"][&introduction]["tf"], 1.0);

    let docs = &index["index"]["documentStore"]["docs"];
    assert_eq!(docs[&first_chapter]["body"], "more text.");
    assert_eq!(docs[&some_section]["body"], "");
    assert_eq!(
        docs[&summary]["body"],
        "Introduction First Chapter Includes Unicode No Headers Duplicate Headers Heading Attributes"
    );
    assert_eq!(
        docs[&summary]["breadcrumbs"],
        "First Chapter » Includes » Summary"
    );
    // See note about InlineHtml in search.rs. Ideally the `alert()` part
    // should not be in the index, but we don't have a way to scrub inline
    // html.
    assert_eq!(
        docs[&sneaky]["body"],
        "I put &lt;HTML&gt; in here! Sneaky inline event alert(\"inline\");. But regular inline is indexed."
    );
    assert_eq!(
        docs[&no_headers]["breadcrumbs"],
        "First Chapter » No Headers"
    );
    assert_eq!(
        docs[&duplicate_headers_1]["breadcrumbs"],
        "First Chapter » Duplicate Headers » Header Text"
    );
    assert_eq!(
        docs[&no_headers]["body"],
        "Capybara capybara capybara. Capybara capybara capybara. ThisLongWordIsIncludedSoWeCanCheckThatSufficientlyLongWordsAreOmittedFromTheSearchIndex."
    );
    assert_eq!(
        docs[&heading_attrs]["breadcrumbs"],
        "First Chapter » Heading Attributes » Heading with id and classes"
    );
}

// This test is here to catch any unexpected changes to the search index.
#[test]
fn search_index_hasnt_changed_accidentally() {
    BookTest::from_dir("search/reasonable_search_index").check_file(
        "book/searchindex.js",
        file!["search/reasonable_search_index/expected_index.js"],
    );
}

// Ability to disable search chapters.
#[test]
fn can_disable_individual_chapters() {
    let mut test = BookTest::from_dir("search/disable_search_chapter");
    test.build();
    let index = read_book_index(&test.dir);
    let doc_urls = index["doc_urls"].as_array().unwrap();
    let contains = |path| {
        doc_urls
            .iter()
            .any(|p| p.as_str().unwrap().starts_with(path))
    };
    assert!(contains("second.html"));
    assert!(!contains("second/"));
    assert!(!contains("first/disable_me.html"));
    assert!(contains("first/keep_me.html"));
}

// Test for a regression where search would fail if source_path is None.
// https://github.com/rust-lang/mdBook/pull/2550
#[test]
fn with_no_source_path() {
    let test = BookTest::from_dir("search/reasonable_search_index");
    let mut book = test.load_book();
    let chapter = Chapter {
        name: "Sample chapter".to_string(),
        content: "".to_string(),
        number: None,
        sub_items: Vec::new(),
        path: Some(PathBuf::from("sample.html")),
        source_path: None,
        parent_names: Vec::new(),
    };
    book.book.sections.push(BookItem::Chapter(chapter));
    book.build().unwrap();
}

// Checks that invalid settings in search chapter is rejected.
#[test]
fn chapter_settings_validation_error() {
    BookTest::from_dir("search/chapter_settings_validation_error").run("build", |cmd| {
        cmd.expect_failure().expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: Rendering failed
[TIMESTAMP] [ERROR] (mdbook_core::utils): [TAB]Caused By: [output.html.search.chapter] key `does-not-exist` does not match any chapter paths

"#]]);
    });
}
