mod dummy_book;

use crate::dummy_book::DummyBook;

use mdbook::MDBook;

#[test]
fn mdbook_test_chapter_not_found() {
    let temp = DummyBook::new().with_passing_test(true).build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();

    assert!(md.test_chapter(vec![], Some("Bogus Chapter Name")).is_err());
}
