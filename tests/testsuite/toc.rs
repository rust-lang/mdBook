//! Tests for table of contents (sidebar).

use crate::prelude::*;
use anyhow::Result;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};

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
    let html = read_to_string(toc_path);
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
<span class="chapter-link-wrapper">
<a href="formatted-summary.html">
<strong aria-hidden="true">1.</strong> Italic code *escape* `escape2`</a>
</span>
</li>
<li class="chapter-item expanded ">
<span class="chapter-link-wrapper">
<a href="soft.html">
<strong aria-hidden="true">2.</strong> Soft line break</a>
</span>
</li>
<li class="chapter-item expanded ">
<span class="chapter-link-wrapper">
<a href="escaped-tag.html">
<strong aria-hidden="true">3.</strong> &lt;escaped tag&gt;</a>
</span>
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

#[test]
#[cfg(not(windows))]
fn percent_encoded_chapter_paths() {
    let mut test = BookTest::from_dir("toc/percent_encoded_chapter_paths");
    // `?` cannot appear in Windows filenames, so create this source at runtime.
    std::fs::write(
        test.dir.join("src/question?.md"),
        "# Question mark\n\n[Query string](asset.png?raw=1)\n",
    )
    .unwrap();
    test.build();

    test.check_toc_js(str![[r#"
<ol class="chapter">
<li class="chapter-item expanded ">
<span class="chapter-link-wrapper">
<a href="question%3F.html">
<strong aria-hidden="true">1.</strong> Question mark</a>
</span>
</li>
<li class="chapter-item expanded ">
<span class="chapter-link-wrapper">
<a href="spati%C3%ABring.html">
<strong aria-hidden="true">2.</strong> Unicode</a>
</span>
</li>
<li class="chapter-item expanded ">
<span class="chapter-link-wrapper">
<a href="number%23one.html">
<strong aria-hidden="true">3.</strong> Fragment delimiter</a>
</span>
</li>
</ol>
"#]]);

    assert!(test.dir.join("book/question?.html").is_file());
    assert!(test.dir.join("book/spatiëring.html").is_file());
    assert!(test.dir.join("book/number#one.html").is_file());

    let question_html = read_to_string(test.dir.join("book/question?.html"));
    assert!(question_html.contains(r#"href="spati%C3%ABring.html""#));
    assert!(question_html.contains(r#"href="asset.png?raw=1""#));

    #[cfg(feature = "search")]
    {
        let search_index = read_to_string(glob_one(&test.dir, "book/searchindex*.js"));
        assert!(search_index.contains("question%3F.html#question-mark"));
        assert!(search_index.contains("spati%C3%ABring.html#unicode"));
        assert!(search_index.contains("number%23one.html#fragment-delimiter"));
    }
}
