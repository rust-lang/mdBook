//! Tests for table of contents (sidebar).

use crate::prelude::*;
use anyhow::Context;
use anyhow::Result;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::fs;

const TOC_TOP_LEVEL: &[&str] = &[
    "1. With Readme",
    "3. Deep Nest 1",
    "Prefix 1",
    "Prefix 2",
    "Suffix 1",
    "Suffix 2",
];
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

/// Read the TOC fallback (`book/toc.html`) HTML and expose it as a DOM which we
/// can search with the `select` crate
fn toc_fallback_html() -> Result<Document> {
    let mut test = BookTest::from_dir("toc/basic_toc");
    test.build();

    let toc_path = test.dir.join("book").join("toc.html");
    let html = fs::read_to_string(toc_path).with_context(|| "Unable to read index.html")?;
    Ok(Document::from(html.as_str()))
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

#[test]
fn check_first_toc_level() {
    let doc = toc_js_html();
    let mut should_be = Vec::from(TOC_TOP_LEVEL);

    should_be.extend(TOC_SECOND_LEVEL);
    should_be.sort_unstable();

    let pred = descendants!(
        Class("chapter"),
        Name("li"),
        Name("a").and(Class("toggle").not())
    );

    let mut children: Vec<_> = doc
        .find(pred)
        .map(|elem| elem.text().trim().to_string())
        .collect();
    children.sort();

    assert_eq!(children, should_be);
}

#[test]
fn check_spacers() {
    let doc = toc_js_html();
    let should_be = 2;

    let num_spacers = doc
        .find(Class("chapter").descendant(Name("li").and(Class("spacer"))))
        .count();
    assert_eq!(num_spacers, should_be);
}

// don't use target="_parent" in JS
#[test]
fn check_link_target_js() {
    let doc = toc_js_html();

    let num_parent_links = doc
        .find(
            Class("chapter")
                .descendant(Name("li"))
                .descendant(Name("a").and(Attr("target", "_parent"))),
        )
        .count();
    assert_eq!(num_parent_links, 0);
}

// don't use target="_parent" in IFRAME
#[test]
fn check_link_target_fallback() {
    let doc = toc_fallback_html().unwrap();

    let num_parent_links = doc
        .find(
            Class("chapter")
                .descendant(Name("li"))
                .descendant(Name("a").and(Attr("target", "_parent"))),
        )
        .count();
    assert_eq!(
        num_parent_links,
        TOC_TOP_LEVEL.len() + TOC_SECOND_LEVEL.len()
    );
}

// Checks formatting of summary names with inline elements.
#[test]
fn summary_with_markdown_formatting() {
    BookTest::from_dir("toc/summary_with_markdown_formatting")
        .check_toc_js(str![[r#"
<ol class="chapter">
<li class="chapter-item expanded ">
<a href="formatted-summary.html">
<strong aria-hidden="true">1.</strong> Italic code *escape* `escape2`</a>
</li>
<li class="chapter-item expanded ">
<a href="soft.html">
<strong aria-hidden="true">2.</strong> Soft line break</a>
</li>
<li class="chapter-item expanded ">
<a href="escaped-tag.html">
<strong aria-hidden="true">3.</strong> &lt;escaped tag&gt;</a>
</li>
</ol>
"#]])
        .check_file(
            "src/formatted-summary.md",
            str![[r#"
# Italic code *escape* `escape2`

"#]],
        )
        .check_file(
            "src/soft.md",
            str![[r#"
# Soft line break

"#]],
        )
        .check_file(
            "src/escaped-tag.md",
            str![[r#"
# &lt;escaped tag&gt;

"#]],
        );
}
