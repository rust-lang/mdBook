#![cfg(test)]

use std::path::PathBuf;
use book::bookconfig::BookConfig;

#[test]
fn it_creates_paths_joined_to_project_root() {
    let result = BookConfig::new(&PathBuf::from("./there".to_string()));

    let mut expected = BookConfig::default();
    expected.dest = PathBuf::from("./there".to_string()).join("book");
    expected.src = PathBuf::from("./there".to_string()).join("src");

    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}
