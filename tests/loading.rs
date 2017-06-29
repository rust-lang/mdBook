//! Integration tests for loading a book into memory

extern crate mdbook;
extern crate env_logger;

use std::path::PathBuf;
use mdbook::loader::{parse_summary, Link, SummaryItem, SectionNumber, Summary};


const SUMMARY: &str = "
# Summary

[Introduction](/intro.md)

---

[A Prefix Chapter](/some_prefix.md)

- [First chapter](/chapter_1/index.md)
  - [Some Subsection](/chapter_1/subsection.md)

---

[Conclusion](/conclusion.md)
";

#[test]
fn parse_summary_md() {
    env_logger::init().unwrap();

    let should_be = Summary {
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
            SummaryItem::Link( Link {
                name: String::from("Conclusion"),
                location: PathBuf::from("/conclusion.md"),
                number: None,
                nested_items: vec![],
            })
        ],
    };

    let got = parse_summary(SUMMARY).unwrap();
    println!("{:#?}", got);

    assert_eq!(got, should_be);
}
