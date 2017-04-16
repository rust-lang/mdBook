extern crate mdbook;
extern crate tempdir;
extern crate dir_diff;

use mdbook::MDBook;
use tempdir::TempDir;

use std::path::Path;

#[test]
fn end_to_end() {
    let book = MDBook::new(Path::new("book-example")).read_config();
    let tmp_dir = TempDir::new("book-example").expect("create temp dir failed");

    let mut book = book.set_dest(tmp_dir.path());

    book.build().expect("book failed to build");

    assert!(!dir_diff::is_different(tmp_dir.path(), "tests/book").unwrap());
}