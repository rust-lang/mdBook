//! Tests for the `mdbook test` command.

use crate::prelude::*;

// Simple test for passing tests.
#[test]
fn passing_tests() {
    BookTest::from_dir("test/passing_tests").run("test", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
 INFO Testing all chapters...

"#]]);
    });
}

// Test for a test failure
#[test]
fn failing_tests() {
    BookTest::from_dir("test/failing_tests").run("test", |cmd| {
        cmd.expect_code(101)
            .expect_stdout(str![[""]])
            // This redacts a large number of lines that come from rustdoc and
            // libtest. If the output from those ever changes, then it would not
            // make it possible to test against different versions of Rust. This
            // still includes a little bit of output, so if that is a problem,
            // add more redactions.
            .expect_stderr(str![[r#"
 INFO Testing all chapters...
ERROR rustdoc returned an error:
args: [..] "failing.md"
--- stdout

running 1 test
test failing.md - Failing_Tests (line 3) ... FAILED

failures:

---- failing.md - Failing_Tests (line 3) stdout ----
Test executable failed ([EXIT_STATUS]: 101).

stderr:

thread [..] panicked at [..]:
fail
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace



failures:
    failing.md - Failing_Tests (line 3)

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in [..]
...
--- stderr

ERROR rustdoc returned an error:
args: [..] "failing_include.md"
--- stdout

running 1 test
test failing_include.md - Failing_Include (line 3) ... FAILED

failures:

---- failing_include.md - Failing_Include (line 3) stdout ----
Test executable failed ([EXIT_STATUS]: 101).

stderr:

thread [..] panicked at [..]:
failing!
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace



failures:
    failing_include.md - Failing_Include (line 3)

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in [..]
...
--- stderr

ERROR One or more tests failed

"#]]);
    });
}

// Test with a specific chapter.
#[test]
fn test_individual_chapter() {
    let mut test = BookTest::from_dir("test/passing_tests");
    test.run("test -c", |cmd| {
        cmd.args(&["Passing 1"])
            .expect_stdout(str![[""]])
            .expect_stderr(str![[r#"
 INFO Testing chapter '"Passing 1"'

"#]]);
    })
    // Can also be a source path.
    .run("test -c passing2.md", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
 INFO Testing chapter '"Passing 2"'

"#]]);
    });
}

// Unknown chapter name.
#[test]
fn chapter_not_found() {
    BookTest::from_dir("test/passing_tests").run("test -c bogus", |cmd| {
        cmd.expect_failure()
            .expect_stdout(str![[""]])
            .expect_stderr(str![[r#"
ERROR Chapter not found: bogus

"#]]);
    });
}
