//! Tests for the `mdbook test` command.

use crate::prelude::*;

// Simple test for passing tests.
#[test]
fn passing_tests() {
    BookTest::from_dir("test/passing_tests").run("test", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
 INFO Testing chapter 'Intro': "intro.md"
 INFO Testing chapter 'Passing 1': "passing1.md"
 INFO Testing chapter 'Passing 2': "passing2.md"

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
 INFO Testing chapter 'Failing Tests': "failing.md"
ERROR rustdoc returned an error:

--- stdout

...
test failing.md - Failing_Tests (line 3) ... FAILED
...
thread [..] panicked at failing.md:3:1:
fail
...
 INFO Testing chapter 'Failing Include': "failing_include.md"
ERROR rustdoc returned an error:

--- stdout
...
test failing_include.md - Failing_Include (line 3) ... FAILED
...
thread [..] panicked at failing_include.md:3:1:
failing!
...
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
 INFO Testing chapter 'Passing 1': "passing1.md"

"#]]);
    })
    // Can also be a source path.
    .run("test -c passing2.md", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
 INFO Testing chapter 'Passing 2': "passing2.md"

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
