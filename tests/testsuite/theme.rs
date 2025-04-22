//! Tests for theme handling.

use crate::prelude::*;

// Checks what happens if the theme directory is missing.
#[test]
fn missing_theme() {
    BookTest::from_dir("theme/missing_theme")
    .run("build", |cmd| {
cmd.expect_failure()
        .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook::book): Book building has started
[TIMESTAMP] [INFO] (mdbook::book): Running the html backend
[TIMESTAMP] [ERROR] (mdbook::utils): Error: Rendering failed
[TIMESTAMP] [ERROR] (mdbook::utils): [TAB]Caused By: theme dir [ROOT]/./non-existent-directory does not exist

"#]]);
    });
}

// Checks what happens if the theme directory is empty.
#[test]
fn empty_theme() {
    BookTest::from_dir("theme/empty_theme").run("build", |cmd| {
        std::fs::create_dir(cmd.dir.join("theme")).unwrap();
        cmd.expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook::book): Book building has started
[TIMESTAMP] [INFO] (mdbook::book): Running the html backend

"#]]);
    });
}

// Checks overriding index.hbs.
#[test]
fn override_index() {
    BookTest::from_dir("theme/override_index").check_file(
        "book/index.html",
        str![[r#"
This is a modified index.hbs!

"#]],
    );
}
