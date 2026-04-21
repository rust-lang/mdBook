use crate::prelude::BookTest;

// Simple smoke test that mdbookignore works.
#[test]
fn ignore_file_is_respected() {
    let mut test = BookTest::from_dir("mdbookignore/simple");
    test.run("build", |_| ());

    assert!(test.dir.join("book/index.html").exists());
    assert!(test.dir.join("book/normal_file").exists());
    assert!(!test.dir.join("book/ignored_file").exists());
    assert!(!test.dir.join("book/.mdbookignore").exists());
}
