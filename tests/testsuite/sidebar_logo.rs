use crate::prelude::*;

#[test]
fn bad_logo_path() {
    BookTest::from_dir("sidebar/logo/not_found").run("build", |cmd| {
        cmd.expect_failure();
        cmd.expect_stderr(
            "ERROR invalid value for `logo`: [ROOT]/src/does/not/exist does not exist\n",
        );
    });
}

#[test]
fn outside_src_dir() {
    BookTest::from_dir("sidebar/logo/outside").run("build", |cmd| {
        cmd.expect_failure();
        cmd.expect_stderr("ERROR invalid value for `logo`: should live under `src/`\n");
    });
}

#[test]
fn abs_path() {
    BookTest::from_dir("sidebar/logo/absolute").run("build", |cmd| {
        cmd.expect_failure();
        cmd.expect_stderr("ERROR invalid value for `logo`: should live under `src/`\n");
    });
}

#[test]
fn good_logo() {
    BookTest::from_dir("sidebar/logo/good")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_file_contains("book/logo.svg", "<svg");
}

#[test]
fn logo_from_other_src() {
    BookTest::from_dir("sidebar/logo/other_src")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_file_contains("book/logo.svg", "<svg");
}
