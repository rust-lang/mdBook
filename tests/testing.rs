extern crate mdbook;

mod dummy_book;

use dummy_book::DummyBook;
use mdbook::MDBook;


#[test]
fn mdbook_can_correctly_test_a_passing_book() {
    let temp = DummyBook::new().with_passing_test(true).build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();

    assert!(md.test(vec![]).is_ok());
}

#[test]
fn mdbook_detects_book_with_failing_tests() {
    let temp = DummyBook::new().with_passing_test(false).build().unwrap();
    let mut md: MDBook = MDBook::load(temp.path()).unwrap();

    assert!(md.test(vec![]).is_err());
}
