//! General build tests.
//!
//! More specific tests should usually go into a module based on the feature.
//! This module should just have general build tests, or misc small things.

use crate::prelude::*;

// Simple smoke test that building works.
#[test]
fn basic_build() {
    BookTest::from_dir("build/basic_build").run("build", |cmd| {
        cmd.expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook::book): Book building has started
[TIMESTAMP] [INFO] (mdbook::book): Running the html backend

"#]]);
    });
}

// Ensure building fails if `create-missing` is false and one of the files does
// not exist.
#[test]
fn failure_on_missing_file() {
    BookTest::from_dir("build/missing_file").run("build", |cmd| {
        cmd.expect_failure().expect_stderr(str![[r#"
[TIMESTAMP] [ERROR] (mdbook::utils): Error: Chapter file not found, ./chapter_1.md
[TIMESTAMP] [ERROR] (mdbook::utils): [TAB]Caused By: [NOT_FOUND]

"#]]);
    });
}
