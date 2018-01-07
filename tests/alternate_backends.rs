//! Integration tests to make sure alternate backends work.

extern crate mdbook;
extern crate tempdir;

use tempdir::TempDir;
use mdbook::config::Config;
use mdbook::MDBook;

#[test]
fn passing_alternate_backend() {
    let (md, _temp) = dummy_book_with_backend("passing", "true");

    md.build().unwrap();
}

#[test]
fn failing_alternate_backend() {
    let (md, _temp) = dummy_book_with_backend("failing", "false");

    md.build().unwrap_err();
}

#[test]
fn alternate_backend_with_arguments() {
    let (md, _temp) = dummy_book_with_backend("arguments", "echo Hello World!");

    md.build().unwrap();
}

fn dummy_book_with_backend(name: &str, command: &str) -> (MDBook, TempDir) {
    let temp = TempDir::new("mdbook").unwrap();

    let mut config = Config::default();
    config
        .set(format!("output.{}.command", name), command)
        .unwrap();

    let md = MDBook::init(temp.path())
        .with_config(config)
        .build()
        .unwrap();

    (md, temp)
}
