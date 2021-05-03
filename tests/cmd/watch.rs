use crate::cmd::run_cmd;
use crate::dummy_book::{assert_contains_strings, DummyBook};
use mdbook::utils::fs::write_file;
use std::env;

#[test]
fn cmd_watch_help() {
    run_cmd(&["watch", "--help"]).success();
}

#[test]
fn cmd_watch_version() {
    run_cmd(&["watch", "--version"]).success();
}
