use crate::cmd::run_cmd;
use crate::dummy_book::{assert_contains_strings, DummyBook};
use mdbook::utils::fs::write_file;
use std::env;

#[test]
fn cmd_build_help() {
    run_cmd(&["build", "--help"]).success();
}

#[test]
fn cmd_build_version() {
    run_cmd(&["build", "--version"]).success();
}

#[test]
fn cmd_build_with_explicit_dir() {
    let temp = DummyBook::new().build().unwrap();
    assert!(temp.path().exists());

    let dir = temp.path().to_str().unwrap();

    run_cmd(&["build", dir]).success();

    let index_html = temp.path().join("book").join("index.html");

    assert!(index_html.exists());
    assert_contains_strings(index_html, &vec![r#"<title>Dummy Book</title>"#]);
}

#[test]
fn cmd_build_with_dest_dir() {
    let temp = DummyBook::new().build().unwrap();
    assert!(temp.path().exists());

    let dir = temp.path().to_str().unwrap();

    run_cmd(&["build", dir, "-d", "other"]).success();

    let index_html = temp.path().join("other").join("index.html");

    assert!(index_html.exists());
    assert_contains_strings(index_html, &vec![r#"<title>Dummy Book</title>"#]);
}

#[test]
fn cmd_build_with_implicit_config_file() {
    let temp = DummyBook::new().build().unwrap();
    assert!(temp.path().exists());

    let dir = temp.path().to_str().unwrap();

    let book_toml = r#"
    [book]
    title = "implicit"
        "#;

    write_file(&temp.path(), "book.toml", book_toml.as_bytes()).unwrap();

    run_cmd(&["build", dir]).success();

    let index_html = temp.path().join("book").join("index.html");

    assert!(index_html.exists());
    assert_contains_strings(index_html, &vec![r#"<title>Dummy Book - implicit</title>"#]);
}

#[test]
fn cmd_build_with_explicit_config_file() {
    let temp = DummyBook::new().build().unwrap();
    assert!(temp.path().exists());

    let dir = temp.path().to_str().unwrap();

    let book_toml = r#"
    [book]
    title = "explicit"
        "#;

    write_file(&temp.path(), "not-book.toml", book_toml.as_bytes()).unwrap();

    run_cmd(&[
        "build",
        "-c",
        &temp.path().join("not-book.toml").to_str().unwrap(),
        dir,
    ])
    .success();

    let index_html = temp.path().join("book").join("index.html");

    assert!(index_html.exists());
    assert_contains_strings(index_html, &vec![r#"<title>Dummy Book - explicit</title>"#]);
}

#[test]
#[ignore]
fn cmd_build_with_implicit_dir() {
    let temp = DummyBook::new().build().unwrap();
    assert!(temp.path().exists());

    assert!(env::set_current_dir(&temp).is_ok());
    run_cmd(&["build"]).success();

    let index_html = temp.path().join("book").join("index.html");

    assert!(index_html.exists());
    assert_contains_strings(index_html, &vec![r#"<title>Dummy Book</title>"#]);
}
