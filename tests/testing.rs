extern crate tempdir;
extern crate mdbook;

mod helpers;
use mdbook::MDBook;


#[test]
fn mdbook_can_correctly_test_a_passing_book() {
    let temp = helpers::create_book(true);
    let mut md = MDBook::new(temp.path());

    assert!(md.test(vec![]).is_ok());
}

#[test]
fn mdbook_detects_book_with_failing_tests() {
    let temp = helpers::create_book(false);
    let mut md: MDBook = MDBook::new(temp.path());

    assert!(md.test(vec![]).is_err());
}