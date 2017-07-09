/// Tests which exercise the overall application, in particular the `MDBook`
/// initialization and build/rendering process.
///
/// This will create an entire book in a temporary directory using some
/// dummy content.

extern crate mdbook;
extern crate tempdir;
extern crate env_logger;

use std::path::Path;
use std::fs::{self, File};
use std::io::{Read, Write};

use tempdir::TempDir;
use mdbook::MDBook;


const SUMMARY_MD: &'static str = "# Summary

[Introduction](intro.md)

- [First Chapter](./first/index.md)
    - [Nested Chapter](./first/nested.md)
- [Second Chapter](./second.md)

[Conclusion](./conclusion.md)
";

const INTRO: &'static str = "# Introduction

Here's some interesting text...";

const FIRST: &'static str = "# First Chapter

more text.";

const NESTED: &'static str = r#"# Nested Chapter

This file has some testable code.

```rust
assert!($TEST_STATUS);
```"#;

const SECOND: &'static str = "# Second Chapter";

const CONCLUSION: &'static str = "# Conclusion";

#[test]
fn build_the_dummy_book() {
    let temp = create_book(true);
    let mut md = MDBook::new(temp.path());

    md.build().unwrap();
}

#[test]
fn mdbook_can_correctly_test_a_passing_book() {
    let temp = create_book(true);
    let mut md = MDBook::new(temp.path());

    assert!(md.test(vec![]).is_ok());
}

#[test]
fn mdbook_detects_book_with_failing_tests() {
    let temp = create_book(false);
    let mut md: MDBook = MDBook::new(temp.path());

    assert!(md.test(vec![]).is_err());
}

#[test]
fn by_default_mdbook_generates_rendered_content_in_the_book_directory() {
    let temp = create_book(false);
    let mut md = MDBook::new(temp.path());

    assert!(!temp.path().join("book").exists());
    md.build().unwrap();

    assert!(temp.path().join("book").exists());
    assert!(temp.path().join("book").join("index.html").exists());
}

#[test]
fn make_sure_bottom_level_files_contain_links_to_chapters() {
    let temp = create_book(false);
    let mut md = MDBook::new(temp.path());
    md.build().unwrap();

    let dest = temp.path().join("book");
    let links = vec![
        "intro.html",
        "first/index.html",
        "first/nested.html",
        "second.html",
        "conclusion.html",
    ];

    let files_in_bottom_dir = vec!["index.html", "intro.html", "second.html", "conclusion.html"];

    for filename in files_in_bottom_dir {
        assert_contains_strings(dest.join(filename), &links);
    }
}

#[test]
fn check_correct_cross_links_in_nested_dir() {
    let temp = create_book(false);
    let mut md = MDBook::new(temp.path());
    md.build().unwrap();

    let first = temp.path().join("book").join("first");
    let links = vec![
        r#"<base href="../">"#,
        "intro.html",
        "first/index.html",
        "first/nested.html",
        "second.html",
        "conclusion.html",
    ];

    let files_in_nested_dir = vec!["index.html", "nested.html"];

    for filename in files_in_nested_dir {
        assert_contains_strings(first.join(filename), &links);
    }
}

#[test]
fn run_mdbook_init() {
    let created_files = vec!["book", "src", "src/SUMMARY.md", "src/chapter_1.md"];

    let temp = TempDir::new("mdbook").unwrap();
    for file in &created_files {
        assert!(!temp.path().join(file).exists());
    }

    let mut md = MDBook::new(temp.path());
    md.init().unwrap();

    for file in &created_files {
        assert!(temp.path().join(file).exists(), "{} doesn't exist", file);
    }
}

#[test]
fn rendered_code_has_playpen_stuff() {
    let temp = create_book(true);
    let mut md = MDBook::new(temp.path());
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

    let temp = create_book(true);
    let mut md = MDBook::new(temp.path());
    md.build().unwrap();

    let destination = temp.path().join("book");

    for (filename, text) in content {
        let path = destination.join(filename);
        assert_contains_strings(path, &[text]);
    }
}

/// Create a dummy book in a temporary directory, using the contents of
/// `SUMMARY_MD` as a guide.
///
/// The "Nested Chapter" file contains a code block with a single
/// `assert!($TEST_STATUS)`. If you want to check MDBook's testing
/// functionality, `$TEST_STATUS` can be substitute for either `true` or
/// `false`. This is done using the `passing_test` parameter.
fn create_book(passing_test: bool) -> TempDir {
    let temp = TempDir::new("dummy_book").unwrap();

    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();

    File::create(src.join("SUMMARY.md"))
        .unwrap()
        .write_all(SUMMARY_MD.as_bytes())
        .unwrap();
    File::create(src.join("intro.md"))
        .unwrap()
        .write_all(INTRO.as_bytes())
        .unwrap();

    let first = src.join("first");
    fs::create_dir_all(&first).unwrap();
    File::create(first.join("index.md"))
        .unwrap()
        .write_all(FIRST.as_bytes())
        .unwrap();

    let to_substitute = if passing_test { "true" } else { "false" };
    let nested_text = NESTED.replace("$TEST_STATUS", to_substitute);
    File::create(first.join("nested.md"))
        .unwrap()
        .write_all(nested_text.as_bytes())
        .unwrap();

    File::create(src.join("second.md"))
        .unwrap()
        .write_all(SECOND.as_bytes())
        .unwrap();
    File::create(src.join("conclusion.md"))
        .unwrap()
        .write_all(CONCLUSION.as_bytes())
        .unwrap();

    temp
}

/// Read the contents of the provided file into memory and then iterate through
/// the list of strings asserting that the file contains all of them.
fn assert_contains_strings<P: AsRef<Path>>(filename: P, strings: &[&str]) {
    println!("Checking {}", filename.as_ref().display());
    println!();

    let mut content = String::new();
    File::open(filename)
        .expect("Couldn't open the provided file")
        .read_to_string(&mut content)
        .unwrap();

    println!("{}", content);
    println!();
    println!();

    for s in strings {
        println!("Checking for {:?}", s);
        assert!(content.contains(s));
    }
}
