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

#[test]
fn ask_the_preprocessor_to_blow_up() {
    let dummy_book = DummyBook::new();
    let temp = dummy_book.build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();
    md.with_preprocessor(example());

    md.config
        .set("preprocessor.nop-preprocessor.blow-up", true)
        .unwrap();

    let got = md.build();

    assert!(got.is_err());
    let error_message = got.err().unwrap().to_string();
    let status = if cfg!(windows) {
        "exit code: 1"
    } else {
        "exit status: 1"
    };
    assert_eq!(
        error_message,
        format!(
            r#"The "nop-preprocessor" preprocessor exited unsuccessfully with {status} status"#
        )
    );
}

#[test]
fn process_the_dummy_book() {
    let dummy_book = DummyBook::new();
    let temp = dummy_book.build().unwrap();
    let mut md = MDBook::load(temp.path()).unwrap();
    md.with_preprocessor(example());

    md.build().unwrap();
}
