use std::env::current_dir;
use std::fs::{read_to_string, remove_dir_all};
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
    let content = read_to_string(".github/workflows/main.yml")
        .expect("failed to read `.github/workflows/main.yml`");
    for line in content.lines() {
        let line = line.trim();
        if let Some(version) = line.strip_prefix("BROWSER_UI_TEST_VERSION:") {
            return version.trim().replace('\'', "");
        }
    }
    panic!("failed to retrieved `browser-ui-test` version");
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

    let mut command = Command::new("npx");
    command
        .arg("browser-ui-test")
        .args(["--variable", "DOC_PATH", book_dir.as_str()])
        .args(["--test-folder", "tests/gui"]);
    if std::env::args().any(|arg| arg == "--disable-headless-test") {
        command.arg("--no-headless");
    }

    // Then we run the GUI tests on it.
    let status = command.status().expect("failed to get command output");
    assert!(status.success(), "{status:?}");
}
