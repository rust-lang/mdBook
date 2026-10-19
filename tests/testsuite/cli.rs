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

// The test command compiles and runs code samples, so its help text should
// describe both stages of its behavior.
#[test]
fn test_help() {
    BookTest::empty().run("test --help", |cmd| {
        cmd.expect_stdout(concat!(
            "Compiles and runs a book's Rust code samples\n\n",
            "Usage: mdbook[EXE] test [OPTIONS] [dir]\n\n",
            "Arguments:\n",
            "  [dir]  Root directory for the book\n",
            "         (Defaults to the current directory when omitted)\n\n",
            "Options:\n",
            "  -c, --chapter <chapter>",
            "   ",
            "\n",
            "  -L, --library-path <dir>  A comma-separated list of directories to add to the crate search path\n",
            "                            when building tests\n",
            "  -h, --help                Print help\n",
            "  -V, --version             Print version\n",
        ))
        .expect_stderr(str![[""]]);
    });
}
