use crate::prelude::BookTest;

// Simple smoke test that mdbookignore works.
#[test]
fn ignore_file_is_respected() {
    let mut test = BookTest::from_dir("ignore/simple");
    test.run("build", |_| ());

    assert!(test.dir.join("book/index.html").exists());
    assert!(test.dir.join("book/normal_file").exists());
    assert!(!test.dir.join("book/ignored_file").exists());
    // make sure the *.md filtering is still applied properly
    assert!(!test.dir.join("book/chapter_1.md").exists());
    assert!(!test.dir.join("book/SUMMARY.md").exists());
}
