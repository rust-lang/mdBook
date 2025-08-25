//! Tests for book configuration loading.

use crate::prelude::*;

// Test that config can load from environment variable.
#[test]
fn config_from_env() {
    BookTest::from_dir("config/empty")
        .run("build", |cmd| {
            cmd.env("MDBOOK_BOOK__TITLE", "Custom env title");
        })
        .check_file_contains(
            "book/index.html",
            "<title>Chapter 1 - Custom env title</title>",
        );

    // json for some subtable
    //
}

// Test environment config with JSON.
#[test]
fn config_json_from_env() {
    // build table
    BookTest::from_dir("config/empty")
        .run("build", |cmd| {
            cmd.env(
                "MDBOOK_BOOK",
                r#"{"title": "My Awesome Book", "authors": ["Michael-F-Bryan"]}"#,
            );
        })
        .check_file_contains(
            "book/index.html",
            "<title>Chapter 1 - My Awesome Book</title>",
        );

    // book table
    BookTest::from_dir("config/empty")
        .run("build", |cmd| {
            cmd.env("MDBOOK_BUILD", r#"{"build-dir": "alt"}"#);
        })
        .check_file_contains("alt/index.html", "<title>Chapter 1</title>");
}

// Test that a preprocessor receives config set in the environment.
#[test]
fn preprocessor_cfg_from_env() {
    let mut test = BookTest::from_dir("config/empty");
    test.rust_program(
        "cat-to-file",
        r#"
        fn main() {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            std::fs::write("out.txt", s).unwrap();
            println!("{{\"items\": []}}");
        }
        "#,
    )
    .run("build", |cmd| {
        cmd.env(
            "MDBOOK_PREPROCESSOR__CAT_TO_FILE",
            r#"{"command":"./cat-to-file", "array": [1,2,3], "number": 123}"#,
        );
    });
    let out = read_to_string(test.dir.join("out.txt"));
    let (ctx, _book) = mdbook_preprocessor::parse_input(out.as_bytes()).unwrap();
    let cfg: serde_json::Value = ctx.config.get("preprocessor.cat-to-file").unwrap().unwrap();
    assert_eq!(
        cfg,
        serde_json::json!({
            "command": "./cat-to-file",
            "array": [1,2,3],
            "number": 123,
        })
    );
}

// Test that a renderer receives config set in the environment.
#[test]
fn output_cfg_from_env() {
    let mut test = BookTest::from_dir("config/empty");
    test.rust_program(
        "cat-to-file",
        r#"
        fn main() {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            std::fs::write("out.txt", s).unwrap();
        }
        "#,
    )
    .run("build", |cmd| {
        cmd.env(
            "MDBOOK_OUTPUT__CAT_TO_FILE",
            r#"{"command":"./cat-to-file", "array": [1,2,3], "number": 123}"#,
        );
    });
    let out = read_to_string(test.dir.join("book/out.txt"));
    let ctx = mdbook_renderer::RenderContext::from_json(out.as_bytes()).unwrap();
    let cfg: serde_json::Value = ctx.config.get("output.cat-to-file").unwrap().unwrap();
    assert_eq!(
        cfg,
        serde_json::json!({
            "command": "./cat-to-file",
            "array": [1,2,3],
            "number": 123,
        })
    );
}

// An invalid key at the top level.
#[test]
fn bad_config_top_level() {
    BookTest::init(|_| {})
        .change_file("book.toml", "foo = 123")
        .run("build", |cmd| {
            cmd.expect_failure()
                .expect_stdout(str![[""]])
                .expect_stderr(str![[r#"
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: Invalid configuration file
[TIMESTAMP] [ERROR] (mdbook_core::utils): [TAB]Caused By: TOML parse error at line 1, column 1
  |
1 | foo = 123
  | ^^^
unknown field `foo`, expected one of `book`, `build`, `rust`, `output`, `preprocessor`


"#]]);
        });
}

// An invalid table at the top level.
#[test]
fn bad_config_top_level_table() {
    BookTest::init(|_| {})
        .change_file(
            "book.toml",
            "[other]\n\
            foo = 123",
        )
        .run("build", |cmd| {
            cmd.expect_failure()
                .expect_stdout(str![[""]])
                .expect_stderr(str![[r#"
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: Invalid configuration file
[TIMESTAMP] [ERROR] (mdbook_core::utils): [TAB]Caused By: TOML parse error at line 1, column 2
  |
1 | [other]
  |  ^^^^^
unknown field `other`, expected one of `book`, `build`, `rust`, `output`, `preprocessor`


"#]]);
        });
}

// An invalid key in the main book table.
#[test]
fn bad_config_in_book_table() {
    BookTest::init(|_| {})
        .change_file(
            "book.toml",
            "[book]\n\
             title = \"bad-config\"\n\
             foo = 123"
        )
        .run("build", |cmd| {
            cmd.expect_failure()
                .expect_stdout(str![[""]])
                .expect_stderr(str![[r#"
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: Invalid configuration file
[TIMESTAMP] [ERROR] (mdbook_core::utils): [TAB]Caused By: TOML parse error at line 3, column 1
  |
3 | foo = 123
  | ^^^
unknown field `foo`, expected one of `title`, `authors`, `description`, `src`, `language`, `text-direction`


"#]]);
        });
}

// An invalid key in the main rust table.
#[test]
fn bad_config_in_rust_table() {
    BookTest::init(|_| {})
        .change_file(
            "book.toml",
            "[rust]\n\
             title = \"bad-config\"\n",
        )
        .run("build", |cmd| {
            cmd.expect_failure()
                .expect_stdout(str![[""]])
                .expect_stderr(str![[r#"
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: Invalid configuration file
[TIMESTAMP] [ERROR] (mdbook_core::utils): [TAB]Caused By: TOML parse error at line 2, column 1
  |
2 | title = "bad-config"
  | ^^^^^
unknown field `title`, expected `edition`


"#]]);
        });
}
