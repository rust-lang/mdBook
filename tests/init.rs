extern crate mdbook;
extern crate tempdir;

use tempdir::TempDir;
use mdbook::MDBook;


#[test]
fn run_mdbook_init() {
    let created_files = vec!["book", "src", "src/SUMMARY.md", "src/chapter_1.md"];

    let temp = TempDir::new("mdbook").unwrap();
    for file in &created_files {
        assert!(!temp.path().join(file).exists());
    }

    let mut md = MDBook::new(temp.path());
    md.init().unwrap();

    for file in &created_files {
        assert!(temp.path().join(file).exists(), "{} doesn't exist", file);
    }
}

#[test]
fn run_mdbook_init_with_custom_args() {
    let created_files = vec!["out", "in", "in/SUMMARY.md", "in/chapter_1.md"];

    let temp = TempDir::new("mdbook").unwrap();
    for file in &created_files {
        assert!(!temp.path().join(file).exists());
    }

    let mut md = MDBook::new(temp.path())
        .with_source("in")
        .with_destination("out");

    md.init().unwrap();

    for file in &created_files {
        assert!(temp.path().join(file).exists(), "{} doesn't exist", file);
    }
}
