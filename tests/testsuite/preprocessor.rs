//! Tests for custom preprocessors.

use crate::prelude::*;
use mdbook::book::Book;
use mdbook::errors::Result;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
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
[TIMESTAMP] [INFO] (mdbook::book): Book building has started
[TIMESTAMP] [INFO] (mdbook::book): Running the html backend
[TIMESTAMP] [INFO] (mdbook::renderer::html_handlebars::hbs_renderer): Book has been generated into `[ROOT]/book`

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
[TIMESTAMP] [INFO] (mdbook::book): Book building has started
Boom!!1!
[TIMESTAMP] [ERROR] (mdbook::utils): Error: The "nop-preprocessor" preprocessor exited unsuccessfully with [EXIT_STATUS]: 1 status

"#]]);
        });
}

fn example() -> CmdPreprocessor {
    CmdPreprocessor::new(
        "nop-preprocessor".to_string(),
        "cargo run --quiet --example nop-preprocessor --".to_string(),
    )
}

#[test]
fn example_supports_whatever() {
    let cmd = example();

    let got = cmd.supports_renderer("whatever");

    assert_eq!(got, true);
}

#[test]
fn example_doesnt_support_not_supported() {
    let cmd = example();

    let got = cmd.supports_renderer("not-supported");

    assert_eq!(got, false);
}
