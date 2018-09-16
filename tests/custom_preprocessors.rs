extern crate mdbook;

use mdbook::preprocess::{CmdPreprocessor, Preprocessor};

fn example() -> CmdPreprocessor {
    CmdPreprocessor::new("nop-preprocessor".to_string(), "cargo run --example nop-preprocessor --".to_string())
}

#[test]
fn check_if_renderer_is_supported() {
    let cmd = example();

    let got = cmd.supports_renderer("whatever");

    assert_eq!(got, true);
}
