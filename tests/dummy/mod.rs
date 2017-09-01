//! This will create an entire book in a temporary directory using some
//! dummy contents from the `tests/dummy-book/` directory.

// Not all features are used in all test crates, so...
#![allow(dead_code, unused_extern_crates)]

extern crate tempdir;

use std::fs::{create_dir_all, File};
use std::io::Write;

use tempdir::TempDir;


const SUMMARY_MD: &'static str = include_str!("book/SUMMARY.md");
const INTRO: &'static str = include_str!("book/intro.md");
const FIRST: &'static str = include_str!("book/first/index.md");
const NESTED: &'static str = include_str!("book/first/nested.md");
const SECOND: &'static str = include_str!("book/second.md");
const CONCLUSION: &'static str = include_str!("book/conclusion.md");


/// Create a dummy book in a temporary directory, using the contents of
/// `SUMMARY_MD` as a guide.
///
/// The "Nested Chapter" file contains a code block with a single
/// `assert!($TEST_STATUS)`. If you want to check MDBook's testing
/// functionality, `$TEST_STATUS` can be substitute for either `true` or
/// `false`. This is done using the `passing_test` parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct DummyBook {
    passing_test: bool,
}

impl DummyBook {
    /// Create a new `DummyBook` with all the defaults.
    pub fn new() -> DummyBook {
        DummyBook::default()
    }

    /// Whether the doc-test included in the "Nested Chapter" should pass or
    /// fail (it passes by default).
    pub fn with_passing_test(&mut self, test_passes: bool) -> &mut Self {
        self.passing_test = test_passes;
        self
    }

    /// Write a book to a temporary directory using the provided settings.
    ///
    /// # Note
    ///
    /// If this fails for any reason it will `panic!()`. If we can't write to a
    /// temporary directory then chances are you've got bigger problems...
    pub fn build(&self) -> TempDir {
        let temp = TempDir::new("dummy_book").unwrap();

        let src = temp.path().join("src");
        create_dir_all(&src).unwrap();

        let first = src.join("first");
        create_dir_all(&first).unwrap();

        let to_substitute = if self.passing_test { "true" } else { "false" };
        let nested_text = NESTED.replace("$TEST_STATUS", to_substitute);

        let inputs = vec![
            (src.join("SUMMARY.md"), SUMMARY_MD),
            (src.join("intro.md"), INTRO),
            (first.join("index.md"), FIRST),
            (first.join("nested.md"), &nested_text),
            (src.join("second.md"), SECOND),
            (src.join("conclusion.md"), CONCLUSION),
        ];

        for (path, content) in inputs {
            File::create(path)
                .unwrap()
                .write_all(content.as_bytes())
                .unwrap();
        }

        temp
    }
}

impl Default for DummyBook {
    fn default() -> DummyBook {
        DummyBook { passing_test: true }
    }
}
