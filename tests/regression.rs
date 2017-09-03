//! High-level regression testing which will build the example book and *try*
//! to make sure key elements don't get accidentally broken.
//!
//! # Warning
//!
//! These tests will need to be updated every time the example book changes.
//! Hopefully Travis will let you know when that happens.


extern crate mdbook;
#[macro_use]
extern crate pretty_assertions;
extern crate select;
extern crate tempdir;
extern crate walkdir;

mod helpers;

use std::path::Path;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};
use select::document::Document;
use select::predicate::{Class, Descendant, Name, Predicate};


const BOOK_ROOT: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/book-example");
const TOC_TOP_LEVEL: &[&'static str] = &[
    "1. mdBook",
    "2. Command Line Tool",
    "3. Format",
    "4. Rust Library",
    "Contributors",
];
const TOC_SECOND_LEVEL: &[&'static str] = &[
    "2.1. init",
    "2.2. build",
    "2.3. watch",
    "2.4. serve",
    "2.5. test",
    "3.1. SUMMARY.md",
    "3.2. Configuration",
    "3.3. Theme",
    "3.4. MathJax Support",
    "3.5. Rust code specific features",
];
const TOC_THIRD_LEVEL: &[&'static str] = &["3.3.1. index.hbs", "3.3.2. Syntax highlighting"];

/// Apply a series of predicates to some root predicate, where each
/// successive predicate is the descendant of the last one.
macro_rules! descendants {
    ($root:expr, $($child:expr),*) => {
        $root
        $(
            .descendant($child)
        )*
    };
}


/// Make sure that all `*.md` files (excluding `SUMMARY.md`) were rendered
/// and placed in the `book` directory with their extensions set to `*.html`.
#[test]
fn chapter_files_were_rendered_to_html() {
    let temp = helpers::build_example_book();
    let src = Path::new(BOOK_ROOT).join("src");

    let chapter_files = WalkDir::new(&src)
        .into_iter()
        .filter_entry(|entry| entry_ends_with(entry, ".md"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| path.file_name().unwrap() != "SUMMARY");

    for chapter in chapter_files {
        let rendered_location = temp.path()
            .join(chapter.strip_prefix(&src).unwrap())
            .with_extension("html");
        assert!(rendered_location.exists(), "{} doesn't exits", rendered_location.display());
    }
}

fn entry_ends_with(entry: &DirEntry, ending: &str) -> bool {
    entry.file_name().to_string_lossy().ends_with(ending)
}

/// Read the main page (`book/index.html`) and expose it as a DOM which we
/// can search with the `select` crate
fn root_index_html() -> Document {
    let temp = helpers::build_example_book();

    let index_page = temp.path().join("book").join("index.html");
    let html = helpers::read_file(&index_page).unwrap();
    Document::from(html.as_str())
}

#[test]
fn check_third_toc_level() {
    let doc = root_index_html();
    let should_be = TOC_THIRD_LEVEL;

    let pred = descendants!(Class("chapter"), Name("li"), Name("li"), Name("li"), Name("a"));

    let children_of_children_of_children: Vec<String> = doc.find(pred)
        .map(|elem| elem.text().trim().to_string())
        .collect();
    assert_eq!(children_of_children_of_children, should_be);
}

#[test]
fn check_second_toc_level() {
    let doc = root_index_html();
    let mut should_be = Vec::from(TOC_SECOND_LEVEL);

    should_be.extend(TOC_THIRD_LEVEL);
    should_be.sort();

    let pred = descendants!(Class("chapter"), Name("li"), Name("li"), Name("a"));

    let mut children_of_children: Vec<String> = doc.find(pred)
        .map(|elem| elem.text().trim().to_string())
        .collect();
    children_of_children.sort();

    assert_eq!(children_of_children, should_be);
}

#[test]
fn check_first_toc_level() {
    let doc = root_index_html();
    let mut should_be = Vec::from(TOC_TOP_LEVEL);

    should_be.extend(TOC_SECOND_LEVEL);
    should_be.extend(TOC_THIRD_LEVEL);
    should_be.sort();

    let pred = descendants!(Class("chapter"), Name("li"), Name("a"));

    let mut children: Vec<String> = doc.find(pred)
        .map(|elem| elem.text().trim().to_string())
        .collect();
    children.sort();

    assert_eq!(children, should_be);
}

#[test]
fn check_spacers() {
    let doc = root_index_html();
    let should_be = 1;

    let num_spacers = doc.find(Class("chapter").descendant(Name("li").and(Class("spacer"))))
        .count();
    assert_eq!(num_spacers, should_be);
}
