mod dummy_book;

use crate::dummy_book::DummyBook;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook::MDBook;

fn example() -> CmdPreprocessor {
    CmdPreprocessor::new(
        "nop-preprocessor".to_string(),
        "cargo run --example nop-preprocessor --".to_string(),
    )
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
