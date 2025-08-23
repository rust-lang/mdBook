//! Tests for custom preprocessors.

use crate::prelude::*;
use anyhow::Result;
use mdbook_core::book::Book;
use mdbook_driver::builtin_preprocessors::CmdPreprocessor;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
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
