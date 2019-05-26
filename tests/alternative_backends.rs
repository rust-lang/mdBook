//! Integration tests to make sure alternative backends work.

use mdbook::config::Config;
use mdbook::MDBook;
#[cfg(not(windows))]
use std::path::Path;
use tempfile::{Builder as TempFileBuilder, TempDir};

#[test]
fn passing_alternate_backend() {
    let (md, _temp) = dummy_book_with_backend("passing", success_cmd());

    md.build().unwrap();
}

#[test]
fn failing_alternate_backend() {
    let (md, _temp) = dummy_book_with_backend("failing", fail_cmd());

    md.build().unwrap_err();
}

#[test]
fn missing_backends_arent_fatal() {
    let (md, _temp) = dummy_book_with_backend("missing", "trduyvbhijnorgevfuhn");

    assert!(md.build().is_ok());
}

#[test]
fn alternate_backend_with_arguments() {
    let (md, _temp) = dummy_book_with_backend("arguments", "echo Hello World!");

    md.build().unwrap();
}

/// Get a command which will pipe `stdin` to the provided file.
#[cfg(not(windows))]
fn tee_command<P: AsRef<Path>>(out_file: P) -> String {
    let out_file = out_file.as_ref();

    if cfg!(windows) {
        format!("cmd.exe /c \"type > {}\"", out_file.display())
    } else {
        format!("tee {}", out_file.display())
    }
}

#[test]
#[cfg(not(windows))]
fn backends_receive_render_context_via_stdin() {
    use mdbook::renderer::RenderContext;
    use std::fs::File;

    let temp = TempFileBuilder::new().prefix("output").tempdir().unwrap();
    let out_file = temp.path().join("out.txt");
    let cmd = tee_command(&out_file);

    let (md, _temp) = dummy_book_with_backend("cat-to-file", &cmd);

    assert!(!out_file.exists());
    md.build().unwrap();
    assert!(out_file.exists());

    let got = RenderContext::from_json(File::open(&out_file).unwrap());
    assert!(got.is_ok());
}

fn dummy_book_with_backend(name: &str, command: &str) -> (MDBook, TempDir) {
    let temp = TempFileBuilder::new().prefix("mdbook").tempdir().unwrap();

    let mut config = Config::default();
    config
        .set(format!("output.{}.command", name), command)
        .unwrap();

    let md = MDBook::init(temp.path())
        .with_config(config)
        .build()
        .unwrap();

    (md, temp)
}

fn fail_cmd() -> &'static str {
    if cfg!(windows) {
        r#"cmd.exe /c "exit 1""#
    } else {
        "false"
    }
}

fn success_cmd() -> &'static str {
    if cfg!(windows) {
        r#"cmd.exe /c "exit 0""#
    } else {
        "true"
    }
}
