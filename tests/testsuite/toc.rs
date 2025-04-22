//! Tests for table of contents (sidebar).

use crate::prelude::*;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};

const TOC_SECOND_LEVEL: &[&str] = &[
    "1.1. Nested Index",
    "1.2. Nested two",
    "3.1. Deep Nest 2",
    "3.1.1. Deep Nest 3",
];

/// Apply a series of predicates to some root predicate, where each
/// successive predicate is the descendant of the last one. Similar to how you
/// might do `ul.foo li a` in CSS to access all anchor tags in the `foo` list.
macro_rules! descendants {
    ($root:expr, $($child:expr),*) => {
        $root
        $(
            .descendant($child)
        )*
    };
}

/// Read the TOC (`book/toc.js`) nested HTML and expose it as a DOM which we
/// can search with the `select` crate
fn toc_js_html() -> Document {
    let mut test = BookTest::from_dir("toc/basic_toc");
    test.build();
    let html = test.toc_js_html();
    Document::from(html.as_str())
}

#[test]
fn check_second_toc_level() {
    let doc = toc_js_html();
    let mut should_be = Vec::from(TOC_SECOND_LEVEL);
    should_be.sort_unstable();

    let pred = descendants!(
        Class("chapter"),
        Name("li"),
        Name("li"),
        Name("a").and(Class("toggle").not())
    );

    let mut children_of_children: Vec<_> = doc
        .find(pred)
        .map(|elem| elem.text().trim().to_string())
        .collect();
    children_of_children.sort();

    assert_eq!(children_of_children, should_be);
}
