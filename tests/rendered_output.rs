mod dummy_book;

use crate::dummy_book::{assert_contains_strings, DummyBook};

use mdbook::config::Config;
use mdbook::MDBook;
use pretty_assertions::assert_eq;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

const BOOK_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/dummy_book");

#[test]
fn by_default_mdbook_generates_rendered_content_in_the_book_directory() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();

    assert!(!temp.path().join("book").exists());
    md.build().unwrap();

    assert!(temp.path().join("book").exists());
    let index_file = md.build_dir_for("html").join("index.html");
    assert!(index_file.exists());
}

#[test]
fn check_correct_cross_links_in_nested_dir() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let first = temp.path().join("book").join("first");

    assert_contains_strings(
        first.join("index.html"),
        &[r##"<h2 id="some-section"><a class="header" href="#some-section">"##],
    );

    assert_contains_strings(
        first.join("nested.html"),
        &[r##"<h2 id="some-section"><a class="header" href="#some-section">"##],
    );
}

#[test]
fn chapter_content_appears_in_rendered_document() {
    let content = vec![
        ("index.html", "This file is just here to cause the"),
        ("intro.html", "Here's some interesting text"),
        ("second.html", "Second Chapter"),
        ("first/nested.html", "testable code"),
        ("first/index.html", "more text"),
        ("conclusion.html", "Conclusion"),
    ];

    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let destination = temp.path().join("book");

    for (filename, text) in content {
        let path = destination.join(filename);
        assert_contains_strings(path, &[text]);
    }
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
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| path.file_name().and_then(OsStr::to_str) != Some("SUMMARY.md"));

    for chapter in chapter_files {
        let rendered_location = temp
            .path()
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

#[test]
fn example_book_can_build() {
    let example_book_dir = dummy_book::new_copy_of_example_book().unwrap();

    let md = MDBook::load(example_book_dir.path()).unwrap();

    md.build().unwrap();
}

/// Checks formatting of summary names with inline elements.
#[test]
fn summary_with_markdown_formatting() {
    let temp = DummyBook::new().build().unwrap();
    let mut cfg = Config::default();
    cfg.set("book.src", "summary-formatting").unwrap();
    let md = MDBook::load_with_config(temp.path(), cfg).unwrap();
    md.build().unwrap();

    let rendered_path = temp.path().join("book/toc.js");
    assert_contains_strings(
        rendered_path,
        &[
            r#"<a href="formatted-summary.html"><strong aria-hidden="true">1.</strong> Italic code *escape* `escape2`</a>"#,
            r#"<a href="soft.html"><strong aria-hidden="true">2.</strong> Soft line break</a>"#,
            r#"<a href="escaped-tag.html"><strong aria-hidden="true">3.</strong> &lt;escaped tag&gt;</a>"#,
        ],
    );

    let generated_md = temp.path().join("summary-formatting/formatted-summary.md");
    assert_eq!(
        fs::read_to_string(generated_md).unwrap(),
        "# Italic code *escape* `escape2`\n"
    );
    let generated_md = temp.path().join("summary-formatting/soft.md");
    assert_eq!(
        fs::read_to_string(generated_md).unwrap(),
        "# Soft line break\n"
    );
    let generated_md = temp.path().join("summary-formatting/escaped-tag.md");
    assert_eq!(
        fs::read_to_string(generated_md).unwrap(),
        "# &lt;escaped tag&gt;\n"
    );
}
