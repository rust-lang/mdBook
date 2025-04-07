use std::fs::OpenOptions;
use std::io::Write;

use crate::cli::cmd::mdbook_cmd;
use crate::dummy_book::DummyBook;

use mdbook::config::Config;
use predicates::prelude::*;

/// Run `mdbook init` with `--force` to skip the confirmation prompts
#[test]
fn base_mdbook_init_can_skip_confirmation_prompts() {
    let temp = DummyBook::new().build().unwrap();

    // doesn't exist before
    assert!(!temp.path().join("book").exists());

    let mut cmd = mdbook_cmd();
    cmd.args(["init", "--force"]).current_dir(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\nAll done, no errors...\n"));

    let config = Config::from_disk(temp.path().join("book.toml")).unwrap();
    assert_eq!(config.book.title, None);

    assert!(!temp.path().join(".gitignore").exists());
}

/// Run `mdbook init` with `--title` without git config.
///
/// Regression test for https://github.com/rust-lang/mdBook/issues/2485
#[test]
fn no_git_config_with_title() {
    let temp = DummyBook::new().build().unwrap();

    // doesn't exist before
    assert!(!temp.path().join("book").exists());

    let mut cmd = mdbook_cmd();
    cmd.args(["init", "--title", "Example title"])
        .current_dir(temp.path())
        .env("GIT_CONFIG_GLOBAL", "")
        .env("GIT_CONFIG_NOSYSTEM", "1");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\nAll done, no errors...\n"));

    let config = Config::from_disk(temp.path().join("book.toml")).unwrap();
    assert_eq!(config.book.title.as_deref(), Some("Example title"));
}

use anyhow::Context;
use tempfile::Builder as TempFileBuilder;

#[test]
fn highlight_in_books_without_theme() {
    // empty folder
    let temp = TempFileBuilder::new()
        .prefix("dummy_book-")
        .tempdir()
        .with_context(|| "Unable to create temp directory")
        .unwrap();

    let mut cmd = mdbook_cmd();
    cmd.args(["init", "--title", "Example title"])
        .current_dir(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\nAll done, no errors...\n"));

    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 3);
    assert_eq!(
        std::fs::read_dir(temp.path().join("book")).unwrap().count(),
        0
    );

    // build the book
    let mut cmd = mdbook_cmd();
    cmd.args(["build"]).current_dir(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::is_empty())
        .stderr(
            predicate::str::contains("Book building has started")
                .and(predicate::str::contains("Running the html backend")),
        );

    let highlight_js =
        std::fs::read_to_string(temp.path().join("book").join("highlight.js")).unwrap();
    // these values depend on the version of highlight.js
    assert!(highlight_js.contains("Highlight.js v11.10.0 (git: 366a8bd012)"));
    assert_eq!(highlight_js.len(), 20584);
    assert!(!highlight_js.contains("/*! `rust` grammar compiled for Highlight.js 11.10.0 */"));
    assert!(!highlight_js.contains("/*! `python` grammar compiled for Highlight.js 11.10.0 */"));

    let chapter_file = temp.path().join("src").join("chapter_1.md");
    assert!(chapter_file.exists());

    // Add rust code block to the chapter file
    let mut fh = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&chapter_file)
        .unwrap();

    if let Err(e) = writeln!(
        fh,
        "```rust\nfn main() {{\n    println!(\"Hello, world!\");\n}}\n```\n\n"
    ) {
        eprintln!("Couldn't write to file: {}", e);
    }

    // build the book
    let mut cmd = mdbook_cmd();
    cmd.args(["build"]).current_dir(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::is_empty())
        .stderr(
            predicate::str::contains("Book building has started")
                .and(predicate::str::contains("Running the html backend")),
        );

    let highlight_js =
        std::fs::read_to_string(temp.path().join("book").join("highlight.js")).unwrap();
    // these values depend on the version of highlight.js
    assert!(highlight_js.contains("Highlight.js v11.10.0 (git: 366a8bd012)"));
    assert_eq!(highlight_js.len(), 23530);
    assert!(highlight_js.contains("/*! `rust` grammar compiled for Highlight.js 11.10.0 */"));
    assert!(!highlight_js.contains("/*! `python` grammar compiled for Highlight.js 11.10.0 */"));

    // Add python code block to the chapter file
    let mut fh = OpenOptions::new()
        .write(true)
        .append(true)
        .open(chapter_file)
        .unwrap();

    if let Err(e) = writeln!(fh, "```python\nprint(\"Hello, world!\")\n```\n\n") {
        eprintln!("Couldn't write to file: {}", e);
    }

    // build the book
    let mut cmd = mdbook_cmd();
    cmd.args(["build"]).current_dir(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::is_empty())
        .stderr(
            predicate::str::contains("Book building has started")
                .and(predicate::str::contains("Running the html backend")),
        );

    let highlight_js =
        std::fs::read_to_string(temp.path().join("book").join("highlight.js")).unwrap();
    // these values depend on the version of highlight.js
    assert!(highlight_js.contains("Highlight.js v11.10.0 (git: 366a8bd012)"));
    assert_eq!(highlight_js.len(), 27452);
    assert!(highlight_js.contains("/*! `rust` grammar compiled for Highlight.js 11.10.0 */"));
    assert!(highlight_js.contains("/*! `python` grammar compiled for Highlight.js 11.10.0 */"));
}

#[test]
fn highlight_in_books_with_them() {
    // empty folder
    let temp = TempFileBuilder::new()
        .prefix("dummy_book-")
        .tempdir()
        .with_context(|| "Unable to create temp directory")
        .unwrap();

    let mut cmd = mdbook_cmd();
    cmd.args(["init", "--title", "Example title", "--theme"])
        .current_dir(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\nAll done, no errors...\n"));

    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 4);
    assert_eq!(
        std::fs::read_dir(temp.path().join("book")).unwrap().count(),
        0
    );
    let highlight_js_in_theme_folder =
        std::fs::read_to_string(temp.path().join("theme").join("highlight.js")).unwrap();
    assert_eq!(highlight_js_in_theme_folder.len(), 1078246); // the whole file

    // pretend that we have a custom highlight.js file
    let content = "Manually modified highlight.js";
    std::fs::write(temp.path().join("theme").join("highlight.js"), content).unwrap();

    // build the book
    let mut cmd = mdbook_cmd();
    cmd.args(["build"]).current_dir(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::is_empty())
        .stderr(
            predicate::str::contains("Book building has started")
                .and(predicate::str::contains("Running the html backend")),
        );

    // check that the highlight.js file in the book folder is the same as the one in the theme folder
    let highlight_js =
        std::fs::read_to_string(temp.path().join("book").join("highlight.js")).unwrap();
    assert_eq!(highlight_js, content);

    let chapter_file = temp.path().join("src").join("chapter_1.md");
    assert!(chapter_file.exists());

    // Add rust code block to the chapter file
    let mut fh = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&chapter_file)
        .unwrap();

    if let Err(e) = writeln!(
        fh,
        "```rust\nfn main() {{\n    println!(\"Hello, world!\");\n}}\n```\n\n"
    ) {
        eprintln!("Couldn't write to file: {}", e);
    }

    // build the book
    let mut cmd = mdbook_cmd();
    cmd.args(["build"]).current_dir(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::is_empty())
        .stderr(
            predicate::str::contains("Book building has started")
                .and(predicate::str::contains("Running the html backend")),
        );

    let highlight_js =
        std::fs::read_to_string(temp.path().join("book").join("highlight.js")).unwrap();
    assert_eq!(highlight_js, content);
}
