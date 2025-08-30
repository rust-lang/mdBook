//! Tests for custom preprocessors.

use crate::book_test::list_all_files;
use crate::prelude::*;
use anyhow::Result;
use mdbook_core::book::{Book, BookItem, Chapter};
use mdbook_driver::builtin_preprocessors::CmdPreprocessor;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use snapbox::IntoData;
use std::sync::{Arc, Mutex};

struct Spy(Arc<Mutex<Inner>>);

#[derive(Debug, Default)]
struct Inner {
    run_count: usize,
    rendered_with: Vec<String>,
}

impl Preprocessor for Spy {
    fn name(&self) -> &str {
        "dummy"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        let mut inner = self.0.lock().unwrap();
        inner.run_count += 1;
        inner.rendered_with.push(ctx.renderer.clone());
        Ok(book)
    }
}

// Test that preprocessor gets run.
#[test]
fn runs_preprocessors() {
    let test = BookTest::init(|_| {});
    let spy: Arc<Mutex<Inner>> = Default::default();
    let mut book = test.load_book();
    book.with_preprocessor(Spy(Arc::clone(&spy)));
    book.build().unwrap();

    let inner = spy.lock().unwrap();
    assert_eq!(inner.run_count, 1);
    assert_eq!(inner.rendered_with, ["html"]);
}

// No-op preprocessor works.
#[test]
fn nop_preprocessor() {
    BookTest::from_dir("preprocessor/nop_preprocessor").run("build", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

"#]]);
    });
}

// Failing preprocessor generates an error.
#[test]
fn failing_preprocessor() {
    BookTest::from_dir("preprocessor/failing_preprocessor")
        .run("build", |cmd| {
            cmd.expect_failure()
                .expect_stdout(str![[""]])
                .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
Boom!!1!
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: The "nop-preprocessor" preprocessor exited unsuccessfully with [EXIT_STATUS]: 1 status

"#]]);
        });
}

fn example() -> CmdPreprocessor {
    CmdPreprocessor::new(
        "nop-preprocessor".to_string(),
        "cargo run --quiet --example nop-preprocessor --".to_string(),
        std::env::current_dir().unwrap(),
        false,
    )
}

#[test]
fn example_supports_whatever() {
    let cmd = example();

    let got = cmd.supports_renderer("whatever").unwrap();

    assert_eq!(got, true);
}

#[test]
fn example_doesnt_support_not_supported() {
    let cmd = example();

    let got = cmd.supports_renderer("not-supported").unwrap();

    assert_eq!(got, false);
}

// Checks the behavior of a relative path to a preprocessor.
#[test]
fn relative_command_path() {
    let mut test = BookTest::init(|_| {});
    test.rust_program(
        "preprocessors/my-preprocessor",
        r#"
        fn main() {
            let mut args = std::env::args().skip(1);
            if args.next().as_deref() == Some("supports") {
                std::fs::write("support-check", args.next().unwrap()).unwrap();
                return;
            }
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            std::fs::write("preprocessor-ran", "test").unwrap();
            println!("{{\"items\": []}}");
        }
        "#,
    )
    .change_file(
        "book.toml",
        "[preprocessor.my-preprocessor]\n\
         command = 'preprocessors/my-preprocessor'\n",
    )
    .run("build", |cmd| {
        cmd.expect_stdout(str![""]).expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

"#]]);
    })
    .check_file("support-check", "html")
    .check_file("preprocessor-ran", "test")
    // Try again, but outside of the book root to check relative path behavior.
    .rm_r("support-check")
    .rm_r("preprocessor-ran")
    .run("build ..", |cmd| {
        cmd.current_dir(cmd.dir.join("src"))
            .expect_stdout(str![""])
            .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/src/../book`

"#]]);
    })
    .check_file("support-check", "html")
    .check_file("preprocessor-ran", "test");
}

// Preprocessor command is missing.
#[test]
fn missing_preprocessor() {
    BookTest::from_dir("preprocessor/missing_preprocessor").run("build", |cmd| {
        cmd.expect_failure()
            .expect_stdout(str![[""]])
            .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [ERROR] (mdbook_driver): The command `trduyvbhijnorgevfuhn` wasn't found, is the `missing` preprocessor installed? If you want to ignore this error when the `missing` preprocessor is not installed, set `optional = true` in the `[preprocessor.missing]` section of the book.toml configuration file.
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: Unable to run the preprocessor `missing`
[TIMESTAMP] [ERROR] (mdbook_core::utils): [TAB]Caused By: [NOT_FOUND]

"#]]);
    });
}

// Optional missing is not an error.
#[test]
fn missing_optional_not_fatal() {
    BookTest::from_dir("preprocessor/missing_optional_not_fatal").run("build", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [WARN] (mdbook_driver): The command `trduyvbhijnorgevfuhn` for preprocessor `missing` was not found, but is marked as optional.
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

"#]]);
    });
}

// with_preprocessor of an existing name.
#[test]
fn with_preprocessor_same_name() {
    let mut test = BookTest::init(|_| {});
    test.change_file(
        "book.toml",
        "[preprocessor.dummy]\n\
         command = 'mdbook-preprocessor-does-not-exist'\n",
    );
    let spy: Arc<Mutex<Inner>> = Default::default();
    let mut book = test.load_book();
    book.with_preprocessor(Spy(Arc::clone(&spy)));
    // Unfortunately this is unable to capture the output when using the API.
    book.build().unwrap();
    let inner = spy.lock().unwrap();
    assert_eq!(inner.run_count, 1);
    assert_eq!(inner.rendered_with, ["html"]);
}

// Checks that the interface stays backwards compatible. The interface here
// should not be changed to fix a compatibility issue unless there is a
// major-semver version update to mdbook.
//
// Note that this tests both preprocessors and renderers. It's in this module
// for lack of a better location.
#[test]
fn extension_compatibility() {
    // This is here to force you to look at this test if you alter any of
    // these types such as adding new fields/variants. This test should be
    // updated accordingly. For example, new `BookItem` variants should be
    // added to the extension_compatibility book, or new fields should be
    // added to the expected input/output. This is also a check that these
    // should only be changed in a semver-breaking release
    let chapter = Chapter {
        name: "example".to_string(),
        content: "content".to_string(),
        number: None,
        sub_items: Vec::new(),
        path: None,
        source_path: None,
        parent_names: Vec::new(),
    };
    let item = BookItem::Chapter(chapter);
    match &item {
        BookItem::Chapter(_) => {}
        BookItem::Separator => {}
        BookItem::PartTitle(_) => {}
    }
    let items = vec![item];
    let _book = Book { items };

    let mut test = BookTest::from_dir("preprocessor/extension_compatibility");
    // Run it once with the preprocessor disabled so that we can verify
    // that the built book is identical with the preprocessor enabled.
    test.run("build", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [WARN] (mdbook_driver): The command `./my-preprocessor` for preprocessor `my-preprocessor` was not found, but is marked as optional.
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book/html`
[TIMESTAMP] [WARN] (mdbook_driver): The command `./my-preprocessor` for preprocessor `my-preprocessor` was not found, but is marked as optional.
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the my-renderer backend
[TIMESTAMP] [INFO] (mdbook_driver::builtin_renderers): Invoking the "my-renderer" renderer
[TIMESTAMP] [WARN] (mdbook_driver): The command `./my-renderer` for backend `my-renderer` was not found, but is marked as optional.

"#]]);
    });
    let orig_dir = test.dir.join("book.orig");
    let pre_dir = test.dir.join("book");
    std::fs::rename(&pre_dir, &orig_dir).unwrap();

    // **CAUTION** DO NOT modify this value unless this is a major-semver change.
    let book_output = serde_json::json!({
        "items": [
          {
            "Chapter": {
              "content": "# Prefix chapter\n",
              "name": "Prefix chapter",
              "number": null,
              "parent_names": [],
              "path": "prefix.md",
              "source_path": "prefix.md",
              "sub_items": []
            }
          },
          {
            "Chapter": {
              "content": "# Chapter 1\n",
              "name": "Chapter 1",
              "number": [
                1
              ],
              "parent_names": [],
              "path": "chapter_1.md",
              "source_path": "chapter_1.md",
              "sub_items": []
            }
          },
          {
            "Chapter": {
              "content": "",
              "name": "Draft chapter",
              "number": [
                2
              ],
              "parent_names": [],
              "path": null,
              "source_path": null,
              "sub_items": []
            }
          },
          {
            "PartTitle": "Part title"
          },
          {
            "Chapter": {
              "content": "# Part chapter\n",
              "name": "Part chapter",
              "number": [
                3
              ],
              "parent_names": [],
              "path": "part/chapter.md",
              "source_path": "part/chapter.md",
              "sub_items": [
                {
                  "Chapter": {
                    "content": "# Part sub chapter\n",
                    "name": "Part sub chapter",
                    "number": [
                      3,
                      1
                    ],
                    "parent_names": [
                      "Part chapter"
                    ],
                    "path": "part/sub-chapter.md",
                    "source_path": "part/sub-chapter.md",
                    "sub_items": []
                  }
                }
              ]
            }
          },
          "Separator",
          {
            "Chapter": {
              "content": "# Suffix chapter\n",
              "name": "Suffix chapter",
              "number": null,
              "parent_names": [],
              "path": "suffix.md",
              "source_path": "suffix.md",
              "sub_items": []
            }
          }
        ]
    });
    let output_str = serde_json::to_string(&book_output).unwrap();
    // **CAUTION** The only updates allowed here in a semver-compatible
    // release is to add new fields.
    let expected_config = serde_json::json!({
        "book": {
          "authors": [],
          "description": null,
          "language": "en",
          "text-direction": null,
          "title": "extension_compatibility"
        },
        "output": {
          "html": {},
          "my-renderer": {
            "command": "./my-renderer",
            "custom-config": "renderer settings",
            "custom-table": {
              "extra": "xyz"
            },
            "optional": true
          }
        },
        "preprocessor": {
          "my-preprocessor": {
            "command": "./my-preprocessor",
            "custom-config": true,
            "custom-table": {
              "extra": "abc"
            },
            "optional": true
          }
        }
    });

    // **CAUTION** The only updates allowed here in a semver-compatible
    // release is to add new fields. The output should not change.
    let expected_preprocessor_input = serde_json::json!([
        {
            "config": expected_config,
            "mdbook_version": "[VERSION]",
            "renderer": "html",
            "root": "[ROOT]"
        },
        book_output
    ]);
    let expected_renderer_input = serde_json::json!(
        {
            "version": "[VERSION]",
            "root": "[ROOT]",
            "book": book_output,
            "config": expected_config,
            "destination": "[ROOT]/book/my-renderer",
        }
    );

    // This preprocessor writes its input to some files, and writes the
    // hard-coded output specified above.
    test.rust_program(
        "my-preprocessor",
        &r###"
            use std::fs::OpenOptions;
            use std::io::{Read, Write};
            fn main() {
                let mut args = std::env::args().skip(1);
                if args.next().as_deref() == Some("supports") {
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("support-check")
                        .unwrap();
                    let renderer = args.next().unwrap();
                    writeln!(file, "{renderer}").unwrap();
                    if renderer != "html" {
                        std::process::exit(1);
                    }
                    return;
                }
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
                std::fs::write("preprocessor-input", &s).unwrap();
                let output = r##"OUTPUT_REPLACE"##;
                println!("{output}");
            }
            "###
        .replace("OUTPUT_REPLACE", &output_str),
    )
    // This renderer writes its input to a file.
    .rust_program(
        "my-renderer",
        &r#"
            fn main() {
                use std::io::Read;
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
                std::fs::write("renderer-input", &s).unwrap();
            }
        "#,
    )
    .run("build", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book/html`
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the my-renderer backend
[TIMESTAMP] [INFO] (mdbook_driver::builtin_renderers): Invoking the "my-renderer" renderer

"#]]);
    })
    .check_file("support-check", "html\nmy-renderer\n")
    .check_file(
        "preprocessor-input",
        serde_json::to_string(&expected_preprocessor_input)
            .unwrap()
            .is_json(),
    )
    .check_file(
        "book/my-renderer/renderer-input",
        serde_json::to_string(&expected_renderer_input)
            .unwrap()
            .is_json(),
    );
    // Verify both directories have the exact same output.
    test.rm_r("book/my-renderer/renderer-input");
    let orig_files = list_all_files(&orig_dir);
    let pre_files = list_all_files(&pre_dir);
    assert_eq!(orig_files, pre_files);
    for file in &orig_files {
        let orig_path = orig_dir.join(file);
        if orig_path.is_file() {
            let orig = std::fs::read(&orig_path).unwrap();
            let pre = std::fs::read(&pre_dir.join(file)).unwrap();
            test.assert.eq(pre, orig);
        }
    }
}
