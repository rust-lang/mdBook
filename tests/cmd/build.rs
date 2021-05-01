use crate::cmd::run_cmd;
use crate::dummy_book::{assert_contains_strings, DummyBook};
use mdbook::utils::fs::write_file;
use mdbook::MDBook;

#[test]
fn cmd_build() {
    let temp = DummyBook::new().build().unwrap();
    assert!(temp.path().exists());

    run_cmd(&["build", temp.path().to_str().unwrap()]).success();
    let index_html = temp.path().join("book").join("index.html");

    assert_contains_strings(index_html, &vec![r#"<title>Dummy Book</title>"#]);
}
