extern crate mdbook;
#[macro_use]
extern crate pretty_assertions;
extern crate select;
extern crate walkdir;

mod dummy_book;

use dummy_book::{assert_contains_strings, DummyBook};

use std::fs;
use std::path::Path;
use std::ffi::OsStr;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use mdbook::errors::*;
use mdbook::utils::fs::file_to_string;
use mdbook::config::Config;
use mdbook::MDBook;


const BOOK_ROOT: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/dummy_book");
const TOC_TOP_LEVEL: &[&'static str] = &[
    "1. First Chapter",
    "2. Second Chapter",
    "Conclusion",
    "Introduction",
];
const TOC_SECOND_LEVEL: &[&'static str] = &["1.1. Nested Chapter"];

/// Make sure you can load the dummy book and build it without panicking.
#[test]
fn build_the_dummy_book() {
    let temp = DummyBook::new().build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();

    md.build().unwrap();
}

#[test]
fn by_default_mdbook_generates_rendered_content_in_the_book_directory() {
    let temp = DummyBook::new().build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();

    assert!(!temp.path().join("book").exists());
    md.build().unwrap();

    assert!(temp.path().join("book").exists());
    assert!(temp.path().join("book").join("index.html").exists());
}

#[test]
fn make_sure_bottom_level_files_contain_links_to_chapters() {
    let temp = DummyBook::new().build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let dest = temp.path().join("book");
    let links = vec![
        r#"href="intro.html""#,
        r#"href="first/index.html""#,
        r#"href="first/nested.html""#,
        r#"href="second.html""#,
        r#"href="conclusion.html""#,
    ];

    let files_in_bottom_dir = vec!["index.html", "intro.html", "second.html", "conclusion.html"];

    for filename in files_in_bottom_dir {
        assert_contains_strings(dest.join(filename), &links);
    }
}

#[test]
fn check_correct_cross_links_in_nested_dir() {
    let temp = DummyBook::new().build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let first = temp.path().join("book").join("first");
    let links = vec![
        r#"<base href="../">"#,
        r#"href="intro.html""#,
        r#"href="first/index.html""#,
        r#"href="first/nested.html""#,
        r#"href="second.html""#,
        r#"href="conclusion.html""#,
    ];

    let files_in_nested_dir = vec!["index.html", "nested.html"];

    for filename in files_in_nested_dir {
        assert_contains_strings(first.join(filename), &links);
    }

    assert_contains_strings(
        first.join("index.html"),
        &[
            r##"href="first/index.html#some-section" id="some-section""##,
        ],
    );

    assert_contains_strings(
        first.join("nested.html"),
        &[
            r##"href="first/nested.html#some-section" id="some-section""##,
        ],
    );
}

#[test]
fn rendered_code_has_playpen_stuff() {
    let temp = DummyBook::new().build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let nested = temp.path().join("book/first/nested.html");
    let playpen_class = vec![r#"class="playpen""#];

    assert_contains_strings(nested, &playpen_class);

    let book_js = temp.path().join("book/book.js");
    assert_contains_strings(book_js, &[".playpen"]);
}

#[test]
fn chapter_content_appears_in_rendered_document() {
    let content = vec![
        ("index.html", "Here's some interesting text"),
        ("second.html", "Second Chapter"),
        ("first/nested.html", "testable code"),
        ("first/index.html", "more text"),
        ("conclusion.html", "Conclusion"),
    ];

    let temp = DummyBook::new().build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let destination = temp.path().join("book");

    for (filename, text) in content {
        let path = destination.join(filename);
        assert_contains_strings(path, &[text]);
    }
}


/// Apply a series of predicates to some root predicate, where each
/// successive predicate is the descendant of the last one. Similar to how you
/// might do `ul.foo li a` in CSS to access all anchor tags in the `foo` list.
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
    let temp = DummyBook::new().build().unwrap();
    let src = Path::new(BOOK_ROOT).join("src");

    let chapter_files = WalkDir::new(&src)
        .into_iter()
        .filter_entry(|entry| entry_ends_with(entry, ".md"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| path.file_name().and_then(OsStr::to_str) != Some("SUMMARY.md"));

    for chapter in chapter_files {
        let rendered_location = temp.path()
            .join(chapter.strip_prefix(&src).unwrap())
            .with_extension("html");
        assert!(
            rendered_location.exists(),
            "{} doesn't exits",
            rendered_location.display()
        );
    }
}

fn entry_ends_with(entry: &DirEntry, ending: &str) -> bool {
    entry.file_name().to_string_lossy().ends_with(ending)
}

/// Read the main page (`book/index.html`) and expose it as a DOM which we
/// can search with the `select` crate
fn root_index_html() -> Result<Document> {
    let temp = DummyBook::new()
        .build()
        .chain_err(|| "Couldn't create the dummy book")?;
    MDBook::load(temp.path())?
        .build()
        .chain_err(|| "Book building failed")?;

    let index_page = temp.path().join("book").join("index.html");
    let html = file_to_string(&index_page).chain_err(|| "Unable to read index.html")?;

    Ok(Document::from(html.as_str()))
}

#[test]
fn check_second_toc_level() {
    let doc = root_index_html().unwrap();
    let mut should_be = Vec::from(TOC_SECOND_LEVEL);
    should_be.sort();

    let pred = descendants!(Class("chapter"), Name("li"), Name("li"), Name("a"));

    let mut children_of_children: Vec<_> = doc.find(pred)
        .map(|elem| elem.text().trim().to_string())
        .collect();
    children_of_children.sort();

    assert_eq!(children_of_children, should_be);
}

#[test]
fn check_first_toc_level() {
    let doc = root_index_html().unwrap();
    let mut should_be = Vec::from(TOC_TOP_LEVEL);

    should_be.extend(TOC_SECOND_LEVEL);
    should_be.sort();

    let pred = descendants!(Class("chapter"), Name("li"), Name("a"));

    let mut children: Vec<_> = doc.find(pred)
        .map(|elem| elem.text().trim().to_string())
        .collect();
    children.sort();

    assert_eq!(children, should_be);
}

#[test]
fn check_spacers() {
    let doc = root_index_html().unwrap();
    let should_be = 1;

    let num_spacers = doc.find(Class("chapter").descendant(Name("li").and(Class("spacer"))))
        .count();
    assert_eq!(num_spacers, should_be);
}

/// Ensure building fails if `create-missing` is false and one of the files does
/// not exist.
#[test]
fn failure_on_missing_file() {
    let temp = DummyBook::new().build().unwrap();
    fs::remove_file(temp.path().join("src").join("intro.md")).unwrap();

    let mut cfg = Config::default();
    cfg.build.create_missing = false;

    let got = MDBook::load_with_config(temp.path(), cfg);
    assert!(got.is_err());
}

/// Ensure a missing file is created if `create-missing` is true.
#[test]
fn create_missing_file_with_config() {
    let temp = DummyBook::new().build().unwrap();
    fs::remove_file(temp.path().join("src").join("intro.md")).unwrap();

    let mut cfg = Config::default();
    cfg.build.create_missing = true;

    assert!(!temp.path().join("src").join("intro.md").exists());
    let _md = MDBook::load_with_config(temp.path(), cfg).unwrap();
    assert!(temp.path().join("src").join("intro.md").exists());
}

/// This makes sure you can include a Rust file with `{{#playpen example.rs}}`.
/// Specification is in `book-example/src/format/rust.md`
#[test]
fn able_to_include_rust_files_in_chapters() {
    let temp = DummyBook::new().build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let second = temp.path().join("book/second.html");

    let playpen_strings = &[
        r#"class="playpen""#,
        r#"println!(&quot;Hello World!&quot;);"#,
    ];
    assert_contains_strings(second, playpen_strings);
}

#[test]
fn example_book_can_build() {
    let example_book_dir = dummy_book::new_copy_of_example_book().unwrap();

    let mut md = MDBook::load(example_book_dir.path()).unwrap();

    let got = md.build();
    assert!(got.is_ok());
}
