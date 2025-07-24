//! Utility for building and running tests against mdbook.

use mdbook_driver::MDBook;
use mdbook_driver::init::BookBuilder;
use snapbox::IntoData;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

/// Test number used for generating unique temp directory names.
static NEXT_TEST_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy, Eq, PartialEq)]
enum StatusCode {
    Success,
    Failure,
    Code(i32),
}

/// Main helper for driving mdbook tests.
pub struct BookTest {
    /// The temp directory where the test should perform its work.
    pub dir: PathBuf,
    assert: snapbox::Assert,
    /// This indicates whether or not the book has been built.
    built: bool,
}

impl BookTest {
    /// Creates a new test, copying the contents from the given directory into
    /// a temp directory.
    pub fn from_dir(dir: &str) -> BookTest {
        // Copy this test book to a temp directory.
        let dir = Path::new("tests/testsuite").join(dir);
        assert!(dir.exists(), "{dir:?} should exist");
        let tmp = Self::new_tmp();
        mdbook_core::utils::fs::copy_files_except_ext(
            &dir,
            &tmp,
            true,
            Some(&PathBuf::from("book")),
            &[],
        )
        .unwrap_or_else(|e| panic!("failed to copy test book {dir:?} to {tmp:?}: {e:?}"));
        Self::new(tmp)
    }

    /// Creates a new test with an empty temp directory.
    pub fn empty() -> BookTest {
        Self::new(Self::new_tmp())
    }

    /// Creates a new test with the given function to initialize a new book.
    ///
    /// The book itself is not built.
    pub fn init(f: impl Fn(&mut BookBuilder)) -> BookTest {
        let tmp = Self::new_tmp();
        let mut bb = MDBook::init(&tmp);
        f(&mut bb);
        bb.build()
            .unwrap_or_else(|e| panic!("failed to initialize book at {tmp:?}: {e:?}"));
        Self::new(tmp)
    }

    fn new_tmp() -> PathBuf {
        let id = NEXT_TEST_ID.fetch_add(1, Ordering::SeqCst);
        let tmp = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("ts")
            .join(format!("t{id}"));
        if tmp.exists() {
            std::fs::remove_dir_all(&tmp)
                .unwrap_or_else(|e| panic!("failed to remove {tmp:?}: {e:?}"));
        }
        std::fs::create_dir_all(&tmp).unwrap_or_else(|e| panic!("failed to create {tmp:?}: {e:?}"));
        tmp
    }

    fn new(dir: PathBuf) -> BookTest {
        let assert = assert(&dir);
        BookTest {
            dir,
            assert,
            built: false,
        }
    }

    /// Checks the contents of an HTML file that it has the given contents
    /// between the `<main>` tag.
    ///
    /// Normally the contents outside of the `<main>` tag aren't interesting,
    /// and they add a significant amount of noise.
    pub fn check_main_file(&mut self, path: &str, expected: impl IntoData) -> &mut Self {
        if !self.built {
            self.build();
        }
        let full_path = self.dir.join(path);
        let actual = read_to_string(&full_path);
        let start = actual
            .find("<main>")
            .unwrap_or_else(|| panic!("didn't find <main> in:\n{actual}"));
        let end = actual.find("</main>").unwrap();
        let contents = actual[start + 6..end - 7].trim();
        self.assert.eq(contents, expected);
        self
    }

    /// Checks the summary contents of `toc.js` against the expected value.
    pub fn check_toc_js(&mut self, expected: impl IntoData) -> &mut Self {
        if !self.built {
            self.build();
        }
        let inner = self.toc_js_html();
        // Would be nice if this were prettified, but a primitive wrapping will do for now.
        let inner = inner.replace("><", ">\n<");
        self.assert.eq(inner, expected);
        self
    }

    /// Returns the summary contents from `toc.js`.
    pub fn toc_js_html(&self) -> String {
        let full_path = self.dir.join("book/toc.js");
        let actual = read_to_string(&full_path);
        let inner = actual
            .lines()
            .filter_map(|line| {
                let line = line.trim().strip_prefix("this.innerHTML = '")?;
                let line = line.strip_suffix("';")?;
                Some(line)
            })
            .next()
            .expect("should have innerHTML");
        inner.to_string()
    }

    /// Checks that the contents of the given file matches the expected value.
    pub fn check_file(&mut self, path: &str, expected: impl IntoData) -> &mut Self {
        if !self.built {
            self.build();
        }
        let path = self.dir.join(path);
        let actual = read_to_string(&path);
        self.assert.eq(actual, expected);
        self
    }

    /// Checks that the given file contains the given string somewhere.
    pub fn check_file_contains(&mut self, path: &str, expected: &str) -> &mut Self {
        if !self.built {
            self.build();
        }
        let path = self.dir.join(path);
        let actual = read_to_string(&path);
        assert!(
            actual.contains(expected),
            "Did not find {expected:?} in {path:?}\n\n{actual}",
        );
        self
    }

    /// Checks that the given file does not contain the given string anywhere.
    ///
    /// Beware that using this is fragile, as it may be unable to catch
    /// regressions (it can't tell the difference between success, or the
    /// string being looked for changed).
    pub fn check_file_doesnt_contain(&mut self, path: &str, string: &str) -> &mut Self {
        if !self.built {
            self.build();
        }
        let path = self.dir.join(path);
        let actual = read_to_string(&path);
        assert!(
            !actual.contains(string),
            "Unexpectedly found {string:?} in {path:?}\n\n{actual}",
        );
        self
    }

    /// Checks that the list of files at the given path matches the given value.
    pub fn check_file_list(&mut self, path: &str, expected: impl IntoData) -> &mut Self {
        let mut all_paths: Vec<_> = walkdir::WalkDir::new(&self.dir.join(path))
            .into_iter()
            // Skip the outer directory.
            .skip(1)
            .map(|e| {
                e.unwrap()
                    .into_path()
                    .strip_prefix(&self.dir)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/")
            })
            .collect();
        all_paths.sort();
        let actual = all_paths.join("\n");
        self.assert.eq(actual, expected);
        self
    }

    /// Loads an [`MDBook`] from the temp directory.
    pub fn load_book(&self) -> MDBook {
        MDBook::load(&self.dir).unwrap_or_else(|e| panic!("book failed to load: {e:?}"))
    }

    /// Builds the book in the temp directory.
    pub fn build(&mut self) -> &mut Self {
        let book = self.load_book();
        book.build()
            .unwrap_or_else(|e| panic!("book failed to build: {e:?}"));
        self.built = true;
        self
    }

    /// Runs the `mdbook` binary in the temp directory.
    ///
    /// This runs `mdbook` with the given args. The args are split on spaces
    /// (if you need args with spaces, use the `args` method). The given
    /// callback receives a [`BookCommand`] for you to customize how the
    /// executable is run.
    pub fn run(&mut self, args: &str, f: impl Fn(&mut BookCommand)) -> &mut Self {
        let mut cmd = BookCommand {
            assert: self.assert.clone(),
            dir: self.dir.clone(),
            args: split_args(args),
            env: BTreeMap::new(),
            expect_status: StatusCode::Success,
            expect_stderr_data: None,
            expect_stdout_data: None,
        };
        f(&mut cmd);
        cmd.run();
        self
    }

    /// Change a file's contents in the given path.
    pub fn change_file(&mut self, path: impl AsRef<Path>, body: &str) -> &mut Self {
        let path = self.dir.join(path);
        std::fs::write(&path, body).unwrap_or_else(|e| panic!("failed to write {path:?}: {e:?}"));
        self
    }

    /// Builds a Rust program with the given src.
    ///
    /// The given path should be the path where to output the executable in
    /// the temp directory.
    pub fn rust_program(&mut self, path: &str, src: &str) -> &mut Self {
        let rs = self.dir.join(path).with_extension("rs");
        let parent = rs.parent().unwrap();
        if !parent.exists() {
            std::fs::create_dir_all(&parent).unwrap();
        }
        std::fs::write(&rs, src).unwrap_or_else(|e| panic!("failed to write {rs:?}: {e:?}"));
        let status = std::process::Command::new("rustc")
            .arg(&rs)
            .current_dir(&parent)
            .status()
            .expect("rustc should run");
        assert!(status.success());
        self
    }
}

/// A builder for preparing to run the `mdbook` executable.
///
/// By default, it expects the process to succeed.
pub struct BookCommand {
    pub dir: PathBuf,
    assert: snapbox::Assert,
    args: Vec<String>,
    env: BTreeMap<String, Option<String>>,
    expect_status: StatusCode,
    expect_stderr_data: Option<snapbox::Data>,
    expect_stdout_data: Option<snapbox::Data>,
}

impl BookCommand {
    /// Indicates that the process should fail.
    pub fn expect_failure(&mut self) -> &mut Self {
        self.expect_status = StatusCode::Failure;
        self
    }

    /// Indicates the process should fail with the given exit code.
    pub fn expect_code(&mut self, code: i32) -> &mut Self {
        self.expect_status = StatusCode::Code(code);
        self
    }

    /// Verifies that stderr matches the given value.
    pub fn expect_stderr(&mut self, expected: impl snapbox::IntoData) -> &mut Self {
        self.expect_stderr_data = Some(expected.into_data());
        self
    }

    /// Verifies that stdout matches the given value.
    pub fn expect_stdout(&mut self, expected: impl snapbox::IntoData) -> &mut Self {
        self.expect_stdout_data = Some(expected.into_data());
        self
    }

    /// Adds arguments to the command to run.
    pub fn args(&mut self, args: &[&str]) -> &mut Self {
        self.args.extend(args.into_iter().map(|t| t.to_string()));
        self
    }

    /// Specifies an environment variable to set on the executable.
    pub fn env<T: Into<String>>(&mut self, key: &str, value: T) -> &mut Self {
        self.env.insert(key.to_string(), Some(value.into()));
        self
    }

    /// Runs the command, and verifies the output.
    fn run(&mut self) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_mdbook"));
        cmd.current_dir(&self.dir)
            .args(&self.args)
            .env_remove("RUST_LOG")
            // Don't read the system git config which is out of our control.
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", &self.dir)
            .env("GIT_CONFIG_SYSTEM", &self.dir)
            .env_remove("GIT_AUTHOR_EMAIL")
            .env_remove("GIT_AUTHOR_NAME")
            .env_remove("GIT_COMMITTER_EMAIL")
            .env_remove("GIT_COMMITTER_NAME");

        for (k, v) in &self.env {
            match v {
                Some(v) => cmd.env(k, v),
                None => cmd.env_remove(k),
            };
        }

        let output = cmd.output().expect("mdbook should be runnable");
        let stdout = std::str::from_utf8(&output.stdout).expect("stdout is not utf8");
        let stderr = std::str::from_utf8(&output.stderr).expect("stderr is not utf8");
        let render_output = || format!("\n--- stdout\n{stdout}\n--- stderr\n{stderr}");
        match (self.expect_status, output.status.success()) {
            (StatusCode::Success, false) => {
                panic!("mdbook failed, but expected success{}", render_output())
            }
            (StatusCode::Failure, true) => {
                panic!("mdbook succeeded, but expected failure{}", render_output())
            }
            (StatusCode::Code(expected), _) => match output.status.code() {
                Some(actual) => assert_eq!(
                    actual, expected,
                    "process exit code did not match as expected"
                ),
                None => panic!("process exited via signal {:?}", output.status),
            },
            _ => {}
        }
        self.expect_status = StatusCode::Success; // Reset to default.
        if let Some(expect_stderr_data) = &self.expect_stderr_data {
            if let Err(e) = self.assert.try_eq(
                Some(&"stderr"),
                stderr.into_data(),
                expect_stderr_data.clone(),
            ) {
                panic!("{e}");
            }
        }
        if let Some(expect_stdout_data) = &self.expect_stdout_data {
            if let Err(e) = self.assert.try_eq(
                Some(&"stdout"),
                stdout.into_data(),
                expect_stdout_data.clone(),
            ) {
                panic!("{e}");
            }
        }
    }
}

fn split_args(s: &str) -> Vec<String> {
    s.split_whitespace()
        .map(|arg| {
            if arg.contains(&['"', '\''][..]) {
                panic!("shell-style argument parsing is not supported");
            }
            String::from(arg)
        })
        .collect()
}

static LITERAL_REDACTIONS: &[(&str, &str)] = &[
    // Unix message for an entity was not found
    ("[NOT_FOUND]", "No such file or directory (os error 2)"),
    // Windows message for an entity was not found
    (
        "[NOT_FOUND]",
        "The system cannot find the file specified. (os error 2)",
    ),
    (
        "[NOT_FOUND]",
        "The system cannot find the path specified. (os error 3)",
    ),
    ("[NOT_FOUND]", "program not found"),
    // Unix message for exit status
    ("[EXIT_STATUS]", "exit status"),
    // Windows message for exit status
    ("[EXIT_STATUS]", "exit code"),
    ("[TAB]", "\t"),
    ("[EXE]", std::env::consts::EXE_SUFFIX),
];

/// This makes it easier to write regex replacements that are guaranteed to only
/// get compiled once
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

fn assert(root: &Path) -> snapbox::Assert {
    let mut subs = snapbox::Redactions::new();
    subs.insert("[ROOT]", root.to_path_buf()).unwrap();
    subs.insert(
        "[TIMESTAMP]",
        regex!(r"(?m)(?<redacted>20\d\d-\d{2}-\d{2} \d{2}:\d{2}:\d{2})"),
    )
    .unwrap();
    subs.insert("[VERSION]", mdbook_core::MDBOOK_VERSION)
        .unwrap();

    subs.extend(LITERAL_REDACTIONS.into_iter().cloned())
        .unwrap();

    snapbox::Assert::new()
        .action_env(snapbox::assert::DEFAULT_ACTION_ENV)
        .redact_with(subs)
}

/// Helper to read a string from the filesystem.
#[track_caller]
pub fn read_to_string<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("could not read file {path:?}: {e:?}"))
}
