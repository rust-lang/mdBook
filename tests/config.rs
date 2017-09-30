extern crate mdbook;
extern crate tempdir;

use std::fs::File;
use std::io::Write;

use mdbook::MDBook;
use tempdir::TempDir;

// Tests that config values unspecified in the configuration file do not
// overwrite
// values specified earlier.
// #[test]
// fn do_not_overwrite_unspecified_config_values() {
//     let dir = TempDir::new("mdbook").expect("Could not create a temp dir");

//     let book = MDBook::new(dir.path())
//         .with_source("bar")
//         .with_destination("baz")
//         .with_mathjax_support(true);

//     assert_eq!(book.get_root(), dir.path());
//     assert_eq!(book.get_source(), dir.path().join("bar"));
//     assert_eq!(book.get_destination(), dir.path().join("baz"));

//     // Test when trying to read a config file that does not exist
//     let book = book.read_config().expect("Error reading the config file");

//     assert_eq!(book.get_root(), dir.path());
//     assert_eq!(book.get_source(), dir.path().join("bar"));
//     assert_eq!(book.get_destination(), dir.path().join("baz"));
//     assert_eq!(book.get_mathjax_support(), true);

//     // Try with a partial config file
//     let file_path = dir.path().join("book.toml");
// let mut f = File::create(file_path).expect("Could not create config
// file");
// f.write_all(br#"source = "barbaz""#).expect("Could not write to config
// file");
//     f.sync_all().expect("Could not sync the file");

//     let book = book.read_config().expect("Error reading the config file");

//     assert_eq!(book.get_root(), dir.path());
//     assert_eq!(book.get_source(), dir.path().join("barbaz"));
//     assert_eq!(book.get_destination(), dir.path().join("baz"));
//     assert_eq!(book.get_mathjax_support(), true);
// }
>>>>>>> Integration tests pass again
