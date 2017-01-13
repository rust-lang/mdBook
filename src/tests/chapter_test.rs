#[cfg(test)]

use std::path::PathBuf;

use book::bookconfig::Author;
use book::chapter::Chapter;

#[test]
fn it_parses_when_exists() {
    let src_path = PathBuf::from(".").join("src").join("tests").join("chapters");
    let path = PathBuf::from("at-the-mountains-of-madness.md");

    let mut result = Chapter::new("Mountains".to_string(), path.clone());
    result.parse_or_create_using(&src_path);

    let mut expected = Chapter::new("Mountains".to_string(), path.clone());

    // test that the author is parsed from the TOML header
    expected.authors = Some(vec![Author::new("H.P. Lovecraft")]);

    assert!(result.content.unwrap().contains("Nemesis, 1917"));
    assert_eq!(format!("{:?}", result.authors), format!("{:?}", expected.authors));
}
