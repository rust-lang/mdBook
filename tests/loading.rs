//! Integration tests for loading a book into memory

#[macro_use]
extern crate pretty_assertions;
extern crate mdbook;
extern crate env_logger;
extern crate tempdir;

use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

use mdbook::loader::{parse_summary, Link, SummaryItem, SectionNumber, Summary, Loader};
use tempdir::TempDir;


const SUMMARY: &'static str = "
# Summary

[Introduction](/intro.md)

---

[A Prefix Chapter](/some_prefix.md)

- [First Chapter](/chapter_1/index.md)
  - [Some Subsection](/chapter_1/subsection.md)

---

[Conclusion](/conclusion.md)
";

#[test]
fn parse_summary_md() {
    env_logger::init().ok();

    let should_be = expected_summary();
    let got = parse_summary(SUMMARY).unwrap();

    println!("{:#?}", got);
    assert_eq!(got, should_be);
}

#[test]
fn parse_summary_using_loader() {
    env_logger::init().ok();

    let temp = TempDir::new("book").unwrap();
    let summary_md = temp.path().join("SUMMARY.md");

    File::create(&summary_md).unwrap().write_all(SUMMARY.as_bytes()).unwrap();

    let loader = Loader::new(temp.path());

    let got = loader.parse_summary().unwrap();
    let should_be = expected_summary();

    assert_eq!(got, should_be);
}

/// This is what the SUMMARY should be parsed as
fn expected_summary() -> Summary {
    Summary {
        title: Some(String::from("Summary")),

        prefix_chapters: vec![
            SummaryItem::Link(Link {
                name: String::from("Introduction"),
                location: PathBuf::from("/intro.md"),
                number: None,
                nested_items: vec![],
            }),
            SummaryItem::Separator,
            SummaryItem::Link(Link {
                name: String::from("A Prefix Chapter"),
                location: PathBuf::from("/some_prefix.md"),
                number: None,
                nested_items: vec![],
            }),
        ],

        numbered_chapters: vec![
            SummaryItem::Link(Link {
                name: String::from("First Chapter"),
                location: PathBuf::from("/chapter_1/index.md"),
                number: Some(SectionNumber(vec![1])),
                nested_items: vec![
                    SummaryItem::Link(Link {
                        name: String::from("Some Subsection"),
                        location: PathBuf::from("/chapter_1/subsection.md"),
                        number: Some(SectionNumber(vec![1, 1])),
                        nested_items: vec![],
                    }),
                ],
            }),
        ],

        suffix_chapters: vec![
            SummaryItem::Separator,
            SummaryItem::Link(Link {
                name: String::from("Conclusion"),
                location: PathBuf::from("/conclusion.md"),
                number: None,
                nested_items: vec![],
            }),
        ],
    }
}
