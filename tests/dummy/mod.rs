//! This will create an entire book in a temporary directory using some
//! dummy contents from the `tests/dummy-book/` directory.

// Not all features are used in all test crates, so...
#![allow(dead_code, unused_variables, unused_imports, unused_extern_crates)]
extern crate mdbook;
extern crate tempdir;
extern crate walkdir;

use std::path::Path;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::error::Error;

// The funny `self::` here is because we've got an `extern crate ...` and are
// in a submodule
use self::tempdir::TempDir;
use self::walkdir::WalkDir;
use self::mdbook::MDBook;


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
        fs::create_dir_all(&src).unwrap();

        let first = src.join("first");
        fs::create_dir_all(&first).unwrap();

        let to_substitute = if self.passing_test { "true" } else { "false" };
        let nested_text = NESTED.replace("$TEST_STATUS", to_substitute);

        let inputs = vec![(src.join("SUMMARY.md"), SUMMARY_MD),
                          (src.join("intro.md"), INTRO),
                          (first.join("index.md"), FIRST),
                          (first.join("nested.md"), &nested_text),
                          (src.join("second.md"), SECOND),
                          (src.join("conclusion.md"), CONCLUSION)];

        for (path, content) in inputs {
            File::create(path).unwrap()
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


/// Copy the example book to a temporary directory and build it.
pub fn build_example_book() -> TempDir {
    let temp = TempDir::new("mdbook").expect("Couldn't create a temporary directory");
    let book_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("book-example");

    recursive_copy(&book_root, temp.path()).expect("Couldn't copy the book example to a temporary directory");

    let book = MDBook::new(temp.path()).build().expect("Building failed");
    temp
}


/// Recursively copy an entire directory tree to somewhere else (a la `cp -r`).
fn recursive_copy<A: AsRef<Path>, B: AsRef<Path>>(from: A, to: B) -> Result<(), Box<Error>> {
    let from = from.as_ref();
    let to = to.as_ref();

    for entry in WalkDir::new(&from) {
        let entry = entry?;

        let original_location = entry.path();
        let relative = original_location.strip_prefix(&from)?;
        let new_location = to.join(relative);

        if original_location.is_file() {
            if let Some(parent) = new_location.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(&original_location, &new_location)?;
        }
    }

    Ok(())
}

pub fn read_file<P: AsRef<Path>>(filename: P) -> Result<String, Box<Error>> {
    let mut contents = String::new();
    File::open(filename)?.read_to_string(&mut contents)?;

    Ok(contents)
}
