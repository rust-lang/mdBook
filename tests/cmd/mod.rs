mod build;

use assert_cmd::assert::Assert;
use assert_cmd::Command;
use std::ffi;

fn run_cmd<I: IntoIterator<Item = S>, S: AsRef<ffi::OsStr>>(args: I) -> Assert {
    Command::cargo_bin("mdbook")
        .expect("Unable to run mdbook")
        .args(args)
        .assert()
}
