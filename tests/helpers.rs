//! Helpers for tests which exercise the overall application, in particular
//! the `MDBook` initialization and build/rendering process.
//!
//! This will create an entire book in a temporary directory using some
//! dummy content.

#![allow(dead_code, unused_variables, unused_imports)]
extern crate tempdir;

use std::path::Path;
use std::fs::{self, File};
use std::io::{Read, Write};

use tempdir::TempDir;


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


/// Create a dummy book in a temporary directory, using the contents of
/// `SUMMARY_MD` as a guide.
///
/// The "Nested Chapter" file contains a code block with a single
/// `assert!($TEST_STATUS)`. If you want to check MDBook's testing
/// functionality, `$TEST_STATUS` can be substitute for either `true` or
/// `false`. This is done using the `passing_test` parameter.
pub fn create_book(passing_test: bool) -> TempDir {
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
pub fn assert_contains_strings<P: AsRef<Path>>(filename: P, strings: &[&str]) {
    let filename = filename.as_ref();

    let mut content = String::new();
    File::open(&filename)
        .expect("Couldn't open the provided file")
        .read_to_string(&mut content)
        .expect("Couldn't read the file's contents");

    for s in strings {
        assert!(content.contains(s), "Searching for {:?} in {}\n\n{}", s, filename.display(), content);
    }
}
