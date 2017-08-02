extern crate tempdir;
extern crate mdbook;

mod helpers;
use mdbook::MDBook;


#[test]
fn mdbook_can_correctly_test_a_passing_book() {
    let temp = helpers::DummyBook::default()
        .with_passing_test(true)
        .build();
    let mut md = MDBook::new(temp.path());

    assert!(md.test(vec![]).is_ok());
}

#[test]
fn mdbook_detects_book_with_failing_tests() {
    let temp = helpers::DummyBook::default()
        .with_passing_test(false)
        .build();
    let mut md: MDBook = MDBook::new(temp.path());

    assert!(md.test(vec![]).is_err());
}