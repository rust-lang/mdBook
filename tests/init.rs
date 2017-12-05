extern crate mdbook;
extern crate tempdir;

use std::path::PathBuf;
use mdbook::MDBook;
use tempdir::TempDir;


/// Run `mdbook init` in an empty directory and make sure the default files
/// are created.
#[test]
fn base_mdbook_init_should_create_default_content() {
    let created_files = vec!["book", "src", "src/SUMMARY.md", "src/chapter_1.md"];

    let temp = TempDir::new("mdbook").unwrap();
    for file in &created_files {
        assert!(!temp.path().join(file).exists());
    }

    let mut md = MDBook::new(temp.path());
    md.init().unwrap();

    for file in &created_files {
        let target = temp.path().join(file);
        println!("{}", target.display());
        assert!(target.exists(), "{} doesn't exist", file);
    }
}

/// Set some custom arguments for where to place the source and destination
/// files, then call `mdbook init`.
#[test]
fn run_mdbook_init_with_custom_book_and_src_locations() {
    let created_files = vec!["out", "in", "in/SUMMARY.md", "in/chapter_1.md"];

    let temp = TempDir::new("mdbook").unwrap();
    for file in &created_files {
        assert!(!temp.path().join(file).exists(),
                "{} shouldn't exist yet!",
                file);
    }

    let mut md = MDBook::new(temp.path());
    md.config.book.src = PathBuf::from("in");
    md.config.build.build_dir = PathBuf::from("out");

    md.init().unwrap();

    for file in &created_files {
        let target = temp.path().join(file);
        assert!(target.exists(), "{} should have been created by `mdbook init`", file);
    }
}
