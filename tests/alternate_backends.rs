//! Integration tests to make sure alternate backends work.

extern crate mdbook;
extern crate tempdir;

use std::fs::File;
use tempdir::TempDir;
use mdbook::config::Config;
use mdbook::MDBook;
use mdbook::renderer::RenderContext;

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

#[test]
fn backends_receive_render_context_via_stdin() {
    let temp = TempDir::new("output").unwrap();
    let out_file = temp.path().join("out.txt");
    let cmd = format!("tee {}", out_file.display());

    let (md, _temp) = dummy_book_with_backend("cat-to-file", &cmd);

    assert!(!out_file.exists());
    md.build().unwrap();
    assert!(out_file.exists());

    let got = RenderContext::from_json(File::open(&out_file).unwrap());
    assert!(got.is_ok());
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
