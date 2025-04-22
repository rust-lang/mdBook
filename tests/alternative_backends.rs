//! Integration tests to make sure alternative backends work.

use mdbook::config::Config;
use mdbook::MDBook;
use std::fs;
use std::path::Path;
use tempfile::Builder as TempFileBuilder;

#[test]
fn relative_command_path() {
    // Checks behavior of relative paths for the `command` setting.
    let temp = TempFileBuilder::new().prefix("mdbook").tempdir().unwrap();
    let renderers = temp.path().join("renderers");
    fs::create_dir(&renderers).unwrap();
    rust_exe(
        &renderers,
        "myrenderer",
        r#"fn main() {
            std::fs::write("output", "test").unwrap();
        }"#,
    );
    let do_test = |cmd_path| {
        let mut config = Config::default();
        config
            .set("output.html", toml::value::Table::new())
            .unwrap();
        config.set("output.myrenderer.command", cmd_path).unwrap();
        let md = MDBook::init(temp.path())
            .with_config(config)
            .build()
            .unwrap();
        let output = temp.path().join("book/myrenderer/output");
        assert!(!output.exists());
        md.build().unwrap();
        assert!(output.exists());
        fs::remove_file(output).unwrap();
    };
    // Legacy paths work, relative to the output directory.
    if cfg!(windows) {
        do_test("../../renderers/myrenderer.exe");
    } else {
        do_test("../../renderers/myrenderer");
    }
    // Modern path, relative to the book directory.
    do_test("renderers/myrenderer");
}

fn rust_exe(temp: &Path, name: &str, src: &str) {
    let rs = temp.join(name).with_extension("rs");
    fs::write(&rs, src).unwrap();
    let status = std::process::Command::new("rustc")
        .arg(rs)
        .current_dir(temp)
        .status()
        .expect("rustc should run");
    assert!(status.success());
}
