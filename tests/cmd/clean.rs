use crate::cmd::run_cmd;
use crate::dummy_book::{assert_contains_strings, DummyBook};
use mdbook::utils::fs::write_file;
use std::env;

#[test]
fn cmd_clean_help() {
    run_cmd(&["clean", "--help"]).success();
}

#[test]
fn cmd_clean_version() {
    run_cmd(&["clean", "--version"]).success();
}
