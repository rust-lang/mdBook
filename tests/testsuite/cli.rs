//! Basic tests for mdbook's CLI.

use crate::prelude::*;
use snapbox::file;

// Test with no args.
#[test]
#[cfg_attr(
    not(all(feature = "watch", feature = "serve")),
    ignore = "needs all features"
)]
fn no_args() {
    BookTest::empty().run("", |cmd| {
        cmd.expect_code(2)
            .expect_stdout(str![[""]])
            .expect_stderr(file!["cli/no_args.term.svg"]);
    });
}

// Help command.
#[test]
#[cfg_attr(
    not(all(feature = "watch", feature = "serve")),
    ignore = "needs all features"
)]
fn help() {
    BookTest::empty()
        .run("help", |cmd| {
            cmd.expect_stdout(file!["cli/help.term.svg"])
                .expect_stderr(str![[""]]);
        })
        .run("--help", |cmd| {
            cmd.expect_stdout(file!["cli/help.term.svg"])
                .expect_stderr(str![[""]]);
        });
}
