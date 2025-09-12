//! Tests for custom renderers.

use crate::prelude::*;
use anyhow::Result;
use mdbook_renderer::{RenderContext, Renderer};
use snapbox::IntoData;
use std::fs::File;
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
 INFO Book building has started
 INFO Running the failing backend
 INFO Invoking the "failing" renderer
ERROR Renderer exited with non-zero return code.
ERROR Rendering failed
[TAB]Caused by: The "failing" renderer failed

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
 INFO Book building has started
 INFO Running the missing backend
 INFO Invoking the "missing" renderer
ERROR The command `trduyvbhijnorgevfuhn` wasn't found, is the `missing` backend installed? If you want to ignore this error when the `missing` backend is not installed, set `optional = true` in the `[output.missing]` section of the book.toml configuration file.
ERROR Rendering failed
[TAB]Caused by: Unable to run the backend `missing`
[TAB]Caused by: [NOT_FOUND]

"#]]);
    });
}

// Optional missing is not an error.
#[test]
fn missing_optional_not_fatal() {
    BookTest::from_dir("renderer/missing_optional_not_fatal").run("build", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the missing backend
 INFO Invoking the "missing" renderer
 WARN The command `trduyvbhijnorgevfuhn` for backend `missing` was not found, but is marked as optional.

"#]]);
    });
}

// Command can include arguments.
#[test]
fn renderer_with_arguments() {
    BookTest::from_dir("renderer/renderer_with_arguments")
        .rust_program(
            "arguments",
            r#"
            fn main() {
                let args: Vec<_> = std::env::args().skip(1).collect();
                assert_eq!(args, &["arg1", "arg2"]);
                println!("Hello World!");
                use std::io::Read;
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
            }
            "#,
        )
        .run("build", |cmd| {
            cmd.expect_stdout(str![[r#"
Hello World!

"#]])
                .expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the arguments backend
 INFO Invoking the "arguments" renderer

"#]]);
        });
}

// Checks the render context received by the renderer.
#[test]
fn backends_receive_render_context_via_stdin() {
    let mut test = BookTest::from_dir("renderer/backends_receive_render_context_via_stdin");
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
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the cat-to-file backend
 INFO Invoking the "cat-to-file" renderer

"#]]);
    })
    .check_file(
        "book/out.txt",
        str![[r##"
{
  "book": {
    "items": [
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
      }
    ]
  },
  "config": {
    "book": {
      "authors": [],
      "description": null,
      "language": "en",
      "text-direction": null,
      "title": null
    },
    "output": {
      "cat-to-file": {
        "command": "./cat-to-file"
      }
    }
  },
  "destination": "[ROOT]/book",
  "root": "[ROOT]",
  "version": "[VERSION]"
}
"##]]
        .is_json(),
    );

    // Can round-trip.
    let f = File::open(test.dir.join("book/out.txt")).unwrap();
    RenderContext::from_json(f).unwrap();
}

// Verifies that a relative path for the renderer command is relative to the
// book root.
#[test]
fn relative_command_path() {
    let mut test = BookTest::init(|_| {});
    test.rust_program(
        "renderers/myrenderer",
        r#"
        fn main() {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            std::fs::write("output", "test").unwrap();
        }
        "#,
    )
    .change_file(
        "book.toml",
        "[output.myrenderer]\n\
         command = 'renderers/myrenderer'\n",
    )
    .run("build", |cmd| {
        cmd.expect_stdout(str![[""]]).expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the myrenderer backend
 INFO Invoking the "myrenderer" renderer

"#]]);
    })
    .check_file("book/output", "test");
}

// with_renderer of an existing name.
#[test]
fn with_renderer_same_name() {
    let mut test = BookTest::init(|_| {});
    test.change_file(
        "book.toml",
        "[output.dummy]\n\
         command = 'mdbook-renderer-does-not-exist'\n",
    );
    let spy: Arc<Mutex<Inner>> = Default::default();
    let mut book = test.load_book();
    book.with_renderer(Spy(Arc::clone(&spy)));
    // Unfortunately this is unable to capture the output when using the API.
    book.build().unwrap();
    let inner = spy.lock().unwrap();
    assert_eq!(inner.run_count, 1);
}
