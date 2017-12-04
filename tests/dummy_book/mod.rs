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
use mdbook::errors::*;
use mdbook::utils::fs::file_to_string;

// The funny `self::` here is because we've got an `extern crate ...` and are
// in a submodule
use self::tempdir::TempDir;
use self::mdbook::MDBook;
use self::walkdir::WalkDir;


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
        DummyBook { passing_test: true }
    }

    /// Whether the doc-test included in the "Nested Chapter" should pass or
    /// fail (it passes by default).
    pub fn with_passing_test(&mut self, test_passes: bool) -> &mut DummyBook {
        self.passing_test = test_passes;
        self
    }

    /// Write a book to a temporary directory using the provided settings.
    pub fn build(&self) -> Result<TempDir> {
        let temp = TempDir::new("dummy_book").chain_err(|| "Unable to create temp directory")?;

        let dummy_book_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/dummy_book");
        recursive_copy(&dummy_book_root, temp.path()).chain_err(|| {
                                                                     "Couldn't copy files into a \
                                                                      temporary directory"
                                                                 })?;

        let sub_pattern = if self.passing_test { "true" } else { "false" };
        let file_containing_test = temp.path().join("src/first/nested.md");
        replace_pattern_in_file(&file_containing_test, "$TEST_STATUS", sub_pattern)?;

        Ok(temp)
    }
}

fn replace_pattern_in_file(filename: &Path, from: &str, to: &str) -> Result<()> {
    let contents = file_to_string(filename)?;
    File::create(filename)?.write_all(contents.replace(from, to).as_bytes())?;

    Ok(())
}

/// Read the contents of the provided file into memory and then iterate through
/// the list of strings asserting that the file contains all of them.
pub fn assert_contains_strings<P: AsRef<Path>>(filename: P, strings: &[&str]) {
    let filename = filename.as_ref();
    let content = file_to_string(filename).expect("Couldn't read the file's contents");

    for s in strings {
        assert!(content.contains(s),
                "Searching for {:?} in {}\n\n{}",
                s,
                filename.display(),
                content);
    }
}



/// Recursively copy an entire directory tree to somewhere else (a la `cp -r`).
fn recursive_copy<A: AsRef<Path>, B: AsRef<Path>>(from: A, to: B) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    for entry in WalkDir::new(&from) {
        let entry = entry.chain_err(|| "Unable to inspect directory entry")?;

        let original_location = entry.path();
        let relative = original_location.strip_prefix(&from)
                                        .expect("`original_location` is inside the `from` \
                                                 directory");
        let new_location = to.join(relative);

        if original_location.is_file() {
            if let Some(parent) = new_location.parent() {
                fs::create_dir_all(parent).chain_err(|| "Couldn't create directory")?;
            }

            fs::copy(&original_location, &new_location).chain_err(|| {
                                                                      "Unable to copy file contents"
                                                                  })?;
        }
    }

    Ok(())
}

pub fn new_copy_of_example_book() -> Result<TempDir> {
    let temp = TempDir::new("book-example")?;
    
    let book_example = Path::new(env!("CARGO_MANIFEST_DIR")).join("book-example");

    recursive_copy(book_example, temp.path())?;

    Ok(temp)
}