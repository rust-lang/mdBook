//! Helpers for tests which exercise the overall application, in particular
//! the `MDBook` initialization and build/rendering process.


use std::path::Path;
use std::fs::File;
use std::io::Read;


/// Read the contents of the provided file into memory and then iterate through
/// the list of strings asserting that the file contains all of them.
pub fn assert_contains_strings<P: AsRef<Path>>(filename: P, strings: &[&str]) {
    let filename = filename.as_ref();

    let mut content = String::new();
    File::open(&filename).expect("Couldn't open the provided file")
                         .read_to_string(&mut content)
                         .expect("Couldn't read the file's contents");

    for s in strings {
        assert!(content.contains(s),
                "Searching for {:?} in {}\n\n{}",
                s,
                filename.display(),
                content);
    }
}
