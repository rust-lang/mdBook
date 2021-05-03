use crate::cmd::run_cmd;
use crate::dummy_book::{assert_contains_strings, DummyBook};
use mdbook::utils::fs::write_file;
use std::env;

#[test]
fn cmd_init_help() {
    run_cmd(&["init", "--help"]).success();
}

#[test]
fn cmd_init_version() {
    run_cmd(&["init", "--version"]).success();
}
