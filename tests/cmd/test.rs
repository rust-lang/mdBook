use crate::cmd::run_cmd;
use crate::dummy_book::{assert_contains_strings, DummyBook};
use mdbook::utils::fs::write_file;
use std::env;

#[test]
fn cmd_test_help() {
    run_cmd(&["test", "--help"]).success();
}

#[test]
fn cmd_test_version() {
    run_cmd(&["test", "--version"]).success();
}
