extern crate mdbook;

use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook::MDBook;
use std::path::Path;

fn example() -> CmdPreprocessor {
    CmdPreprocessor::new("nop-preprocessor".to_string(), "cargo run --example nop-preprocessor --".to_string())
}

#[test]
fn example_supports_whatever() {
    let cmd = example();

    let got = cmd.supports_renderer("whatever");

    assert_eq!(got, true);
}

#[test]
fn example_doesnt_support_not_supported() {
    let cmd = example();

    let got = cmd.supports_renderer("not-supported");

    assert_eq!(got, false);
}

#[test]
fn ask_the_preprocessor_to_blow_up() {
    let dummy_book = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("dummy_book");
    let mut md = MDBook::load(&dummy_book).unwrap();
    md.with_preprecessor(example());

    md.config.set("preprocess.nop-preprocessor.blow-up", true).unwrap();

    let got = md.build();

    assert!(got.is_err());
}

#[test]
fn process_the_dummy_book() {
    let dummy_book = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("dummy_book");
    let mut md = MDBook::load(&dummy_book).unwrap();
    md.with_preprecessor(example());

    md.build().unwrap();
}
