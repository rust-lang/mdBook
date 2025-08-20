//! The GUI test runner.
//!
//! This uses the browser-ui-test npm package to use a headless Chrome to
//! exercise the behavior of rendered books. See `CONTRIBUTING.md` for more
//! information.

use serde_json::Value;
use std::collections::HashSet;
use std::env::current_dir;
use std::fs::{read_dir, read_to_string, remove_dir_all};
use std::process::Command;

fn get_available_browser_ui_test_version_inner(global: bool) -> Option<String> {
    let mut command = Command::new("npm");
    command
        .arg("list")
        .arg("--parseable")
        .arg("--long")
        .arg("--depth=0");
    if global {
        command.arg("--global");
    }
    let stdout = command.output().expect("`npm` command not found").stdout;
    let lines = String::from_utf8_lossy(&stdout);
    lines
        .lines()
        .find_map(|l| l.split(':').nth(1)?.strip_prefix("browser-ui-test@"))
        .map(std::borrow::ToOwned::to_owned)
}

fn get_available_browser_ui_test_version() -> Option<String> {
    get_available_browser_ui_test_version_inner(false)
        .or_else(|| get_available_browser_ui_test_version_inner(true))
}

fn expected_browser_ui_test_version() -> String {
    let content = read_to_string("package.json").expect("failed to read `package.json`");
    let v: Value = serde_json::from_str(&content).expect("failed to parse `package.json`");
    let Some(dependencies) = v.get("dependencies") else {
        panic!("Missing `dependencies` key in `package.json`");
    };
    let Some(browser_ui_test) = dependencies.get("browser-ui-test") else {
        panic!("Missing `browser-ui-test` key in \"dependencies\" object in `package.json`");
    };
    let Value::String(version) = browser_ui_test else {
        panic!("`browser-ui-test` version is not a string");
    };
    version.trim().to_string()
}

fn main() {
    let browser_ui_test_version = expected_browser_ui_test_version();
    match get_available_browser_ui_test_version() {
        Some(version) => {
            if version != browser_ui_test_version {
                eprintln!(
                    "⚠️ Installed version of browser-ui-test (`{version}`) is different than the \
                     one used in the CI (`{browser_ui_test_version}`) You can install this version \
                     using `npm update browser-ui-test` or by using `npm install browser-ui-test\
                     @{browser_ui_test_version}`",
                );
            }
        }
        None => {
            panic!(
                "`browser-ui-test` is not installed. You can install this package using `npm \
                 update browser-ui-test` or by using `npm install browser-ui-test\
                 @{browser_ui_test_version}`",
            );
        }
    }

    let current_dir = current_dir().expect("failed to retrieve current directory");
    let test_book = current_dir.join("test_book");

    // Result doesn't matter.
    let _ = remove_dir_all(test_book.join("book"));

    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("build").arg(&test_book);
    // Then we run the GUI tests on it.
    assert!(cmd.status().is_ok_and(|status| status.success()));

    let book_dir = format!("file://{}", current_dir.join("test_book/book/").display());

    let mut no_headless = false;
    let mut filters = Vec::new();
    for arg in std::env::args().skip(1) {
        if arg == "--disable-headless-test" {
            no_headless = true;
        } else {
            filters.push(arg);
        }
    }

    let mut command = Command::new("npx");
    command
        .arg("browser-ui-test")
        .args(["--variable", "DOC_PATH", book_dir.as_str()]);
    if no_headless {
        command.arg("--no-headless");
    }

    let test_dir = "tests/gui";
    if filters.is_empty() {
        command.args(["--test-folder", test_dir]);
    } else {
        let files = read_dir(test_dir)
            .map(|dir| {
                dir.filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .filter(|path| {
                        path.extension().is_some_and(|ext| ext == "goml") && path.is_file()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or(Vec::new());
        let mut matches = HashSet::new();
        for filter in filters {
            for file in files.iter().filter(|f| {
                f.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains(&filter))
            }) {
                matches.insert(file.display().to_string());
            }
        }
        if matches.is_empty() {
            println!("No test found");
            return;
        }
        command.arg("--test-files");
        for entry in matches {
            command.arg(entry);
        }
    }

    // Then we run the GUI tests on it.
    let status = command.status().expect("failed to get command output");
    assert!(status.success(), "{status:?}");
}
