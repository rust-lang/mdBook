extern crate mdbook;

mod dummy_book;

use dummy_book::DummyBook;

use mdbook::MDBook;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::book::Book;
use mdbook::config::Config;
use mdbook::errors::*;

use std::sync::{Arc, Mutex};

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

#[test]
fn mdbook_runs_preprocessors() {

    let has_run: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    struct DummyPreprocessor(Arc<Mutex<bool>>);

    impl Preprocessor for DummyPreprocessor {
        fn name(&self) -> &str {
            "dummy"
        }

        fn run(&self, _ctx: &PreprocessorContext, _book: &mut Book) -> Result<()> {
            *self.0.lock().unwrap() = true;
            Ok(())
        }
    }

    let temp = DummyBook::new().build().unwrap();
    let cfg = Config::default();

    let mut book = MDBook::load_with_config(temp.path(), cfg).unwrap();
    book.with_preprecessor(DummyPreprocessor(Arc::clone(&has_run)));
    book.build().unwrap();

    assert!(*has_run.lock().unwrap())
}
