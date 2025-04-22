//! Tests for custom renderers.

use crate::prelude::*;
use mdbook::errors::Result;
use mdbook::renderer::{RenderContext, Renderer};
use std::sync::{Arc, Mutex};

struct Spy(Arc<Mutex<Inner>>);

#[derive(Debug, Default)]
struct Inner {
    run_count: usize,
}

impl Renderer for Spy {
    fn name(&self) -> &str {
        "dummy"
    }

    fn render(&self, _ctx: &RenderContext) -> Result<()> {
        let mut inner = self.0.lock().unwrap();
        inner.run_count += 1;
        Ok(())
    }
}

// Test that renderer gets run.
#[test]
fn runs_renderers() {
    let test = BookTest::init(|_| {});
    let spy: Arc<Mutex<Inner>> = Default::default();
    let mut book = test.load_book();
    book.with_renderer(Spy(Arc::clone(&spy)));
    book.build().unwrap();

    let inner = spy.lock().unwrap();
    assert_eq!(inner.run_count, 1);
}

// Test renderer with a failing command fails.
#[test]
fn failing_command() {
    BookTest::init(|_| {})
        .rust_program(
            "failing",
            r#"
            fn main() {
                // Read from stdin to avoid random pipe failures on Linux.
                use std::io::Read;
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
                std::process::exit(1);
            }
            "#,
        )
        .change_file(
            "book.toml",
            "[output.failing]\n\
             command = './failing'\n",
        )
        .run("build", |cmd| {
            cmd.expect_failure()
                .expect_stdout(str![[""]])
                .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook::book): Book building has started
[TIMESTAMP] [INFO] (mdbook::book): Running the failing backend
[TIMESTAMP] [INFO] (mdbook::renderer): Invoking the "failing" renderer
[TIMESTAMP] [ERROR] (mdbook::renderer): Renderer exited with non-zero return code.
[TIMESTAMP] [ERROR] (mdbook::utils): Error: Rendering failed
[TIMESTAMP] [ERROR] (mdbook::utils): [TAB]Caused By: The "failing" renderer failed

"#]]);
        });
}

// Renderer command is missing.
#[test]
fn missing_renderer() {
    BookTest::from_dir("renderer/missing_renderer").run("build", |cmd| {
        cmd.expect_failure()
            .expect_stdout(str![[""]])
            .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook::book): Book building has started
[TIMESTAMP] [INFO] (mdbook::book): Running the missing backend
[TIMESTAMP] [INFO] (mdbook::renderer): Invoking the "missing" renderer
[TIMESTAMP] [ERROR] (mdbook::renderer): The command `trduyvbhijnorgevfuhn` wasn't found, is the "missing" backend installed? If you want to ignore this error when the "missing" backend is not installed, set `optional = true` in the `[output.missing]` section of the book.toml configuration file.
[TIMESTAMP] [ERROR] (mdbook::utils): Error: Rendering failed
[TIMESTAMP] [ERROR] (mdbook::utils): [TAB]Caused By: Unable to start the backend
[TIMESTAMP] [ERROR] (mdbook::utils): [TAB]Caused By: [NOT_FOUND]

"#]]);
    });
}
