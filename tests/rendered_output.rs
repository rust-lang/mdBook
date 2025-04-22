mod dummy_book;

use crate::dummy_book::{assert_contains_strings, DummyBook};

use anyhow::Context;
use mdbook::config::Config;
use mdbook::errors::*;
use mdbook::utils::fs::write_file;
use mdbook::MDBook;
use pretty_assertions::assert_eq;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tempfile::Builder as TempFileBuilder;
use walkdir::{DirEntry, WalkDir};

const BOOK_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/dummy_book");
const TOC_TOP_LEVEL: &[&str] = &[
    "1. First Chapter",
    "2. Second Chapter",
    "Conclusion",
    "Dummy Book",
    "Introduction",
];
const TOC_SECOND_LEVEL: &[&str] = &[
    "1.1. Nested Chapter",
    "1.2. Includes",
    "1.3. Recursive",
    "1.4. Markdown",
    "1.5. Unicode",
    "1.6. No Headers",
    "1.7. Duplicate Headers",
    "1.8. Heading Attributes",
    "2.1. Nested Chapter",
];

#[test]
fn by_default_mdbook_generates_rendered_content_in_the_book_directory() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();

    assert!(!temp.path().join("book").exists());
    md.build().unwrap();

    assert!(temp.path().join("book").exists());
    let index_file = md.build_dir_for("html").join("index.html");
    assert!(index_file.exists());
}

#[test]
fn check_correct_cross_links_in_nested_dir() {
    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let first = temp.path().join("book").join("first");

    assert_contains_strings(
        first.join("index.html"),
        &[r##"<h2 id="some-section"><a class="header" href="#some-section">"##],
    );

    assert_contains_strings(
        first.join("nested.html"),
        &[r##"<h2 id="some-section"><a class="header" href="#some-section">"##],
    );
}

#[test]
fn chapter_content_appears_in_rendered_document() {
    let content = vec![
        ("index.html", "This file is just here to cause the"),
        ("intro.html", "Here's some interesting text"),
        ("second.html", "Second Chapter"),
        ("first/nested.html", "testable code"),
        ("first/index.html", "more text"),
        ("conclusion.html", "Conclusion"),
    ];

    let temp = DummyBook::new().build().unwrap();
    let md = MDBook::load(temp.path()).unwrap();
    md.build().unwrap();

    let destination = temp.path().join("book");

    for (filename, text) in content {
        let path = destination.join(filename);
        assert_contains_strings(path, &[text]);
    }
}

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

/// Make sure that all `*.md` files (excluding `SUMMARY.md`) were rendered
/// and placed in the `book` directory with their extensions set to `*.html`.
#[test]
fn chapter_files_were_rendered_to_html() {
    let temp = DummyBook::new().build().unwrap();
    let src = Path::new(BOOK_ROOT).join("src");

    let chapter_files = WalkDir::new(&src)
        .into_iter()
        .filter_entry(|entry| entry_ends_with(entry, ".md"))
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| path.file_name().and_then(OsStr::to_str) != Some("SUMMARY.md"));

    for chapter in chapter_files {
        let rendered_location = temp
            .path()
            .join(chapter.strip_prefix(&src).unwrap())
            .with_extension("html");
        assert!(
            rendered_location.exists(),
            "{} doesn't exits",
            rendered_location.display()
        );
    }
}

fn entry_ends_with(entry: &DirEntry, ending: &str) -> bool {
    entry.file_name().to_string_lossy().ends_with(ending)
}

/// Read the TOC (`book/toc.js`) nested HTML and expose it as a DOM which we
/// can search with the `select` crate
fn toc_js_html() -> Result<Document> {
    let temp = DummyBook::new()
        .build()
        .with_context(|| "Couldn't create the dummy book")?;
    MDBook::load(temp.path())?
        .build()
        .with_context(|| "Book building failed")?;

    let toc_path = temp.path().join("book").join("toc.js");
    let html = fs::read_to_string(toc_path).with_context(|| "Unable to read index.html")?;
    for line in html.lines() {
        if let Some(left) = line.strip_prefix("        this.innerHTML = '") {
            if let Some(html) = left.strip_suffix("';") {
                return Ok(Document::from(html));
            }
        }
    }
    panic!("cannot find toc in file")
}

/// Read the TOC fallback (`book/toc.html`) HTML and expose it as a DOM which we
/// can search with the `select` crate
fn toc_fallback_html() -> Result<Document> {
    let temp = DummyBook::new()
        .build()
        .with_context(|| "Couldn't create the dummy book")?;
    MDBook::load(temp.path())?
        .build()
        .with_context(|| "Book building failed")?;

    let toc_path = temp.path().join("book").join("toc.html");
    let html = fs::read_to_string(toc_path).with_context(|| "Unable to read index.html")?;
    Ok(Document::from(html.as_str()))
}

#[test]
fn check_second_toc_level() {
    let doc = toc_js_html().unwrap();
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
    let doc = toc_js_html().unwrap();
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
    let doc = toc_js_html().unwrap();
    let should_be = 2;

    let num_spacers = doc
        .find(Class("chapter").descendant(Name("li").and(Class("spacer"))))
        .count();
    assert_eq!(num_spacers, should_be);
}

// don't use target="_parent" in JS
#[test]
fn check_link_target_js() {
    let doc = toc_js_html().unwrap();

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

#[test]
fn example_book_can_build() {
    let example_book_dir = dummy_book::new_copy_of_example_book().unwrap();

    let md = MDBook::load(example_book_dir.path()).unwrap();

    md.build().unwrap();
}

/// Checks formatting of summary names with inline elements.
#[test]
fn summary_with_markdown_formatting() {
    let temp = DummyBook::new().build().unwrap();
    let mut cfg = Config::default();
    cfg.set("book.src", "summary-formatting").unwrap();
    let md = MDBook::load_with_config(temp.path(), cfg).unwrap();
    md.build().unwrap();

    let rendered_path = temp.path().join("book/toc.js");
    assert_contains_strings(
        rendered_path,
        &[
            r#"<a href="formatted-summary.html"><strong aria-hidden="true">1.</strong> Italic code *escape* `escape2`</a>"#,
            r#"<a href="soft.html"><strong aria-hidden="true">2.</strong> Soft line break</a>"#,
            r#"<a href="escaped-tag.html"><strong aria-hidden="true">3.</strong> &lt;escaped tag&gt;</a>"#,
        ],
    );

    let generated_md = temp.path().join("summary-formatting/formatted-summary.md");
    assert_eq!(
        fs::read_to_string(generated_md).unwrap(),
        "# Italic code *escape* `escape2`\n"
    );
    let generated_md = temp.path().join("summary-formatting/soft.md");
    assert_eq!(
        fs::read_to_string(generated_md).unwrap(),
        "# Soft line break\n"
    );
    let generated_md = temp.path().join("summary-formatting/escaped-tag.md");
    assert_eq!(
        fs::read_to_string(generated_md).unwrap(),
        "# &lt;escaped tag&gt;\n"
    );
}

#[test]
fn custom_fonts() {
    // Tests to ensure custom fonts are copied as expected.
    let actual_files = |path: &Path| -> Vec<String> {
        let mut actual: Vec<_> = path
            .read_dir()
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        actual.sort();
        actual
    };
    let has_fonts_css = |path: &Path| -> bool {
        let contents = fs::read_to_string(path.join("book/index.html")).unwrap();
        contents.contains("fonts/fonts.css")
    };

    // copy-fonts=false with empty fonts.css
    let temp = TempFileBuilder::new().prefix("mdbook").tempdir().unwrap();
    let p = temp.path();
    MDBook::init(p).build().unwrap();
    write_file(&p.join("theme/fonts"), "fonts.css", b"").unwrap();
    let config = Config::from_str("output.html.copy-fonts = false").unwrap();
    MDBook::load_with_config(p, config)
        .unwrap()
        .build()
        .unwrap();
    assert!(!has_fonts_css(p));
    assert!(!p.join("book/fonts").exists());

    // copy-fonts=false with fonts theme
    let temp = TempFileBuilder::new().prefix("mdbook").tempdir().unwrap();
    let p = temp.path();
    MDBook::init(p).build().unwrap();
    write_file(&p.join("theme/fonts"), "fonts.css", b"/*custom*/").unwrap();
    write_file(&p.join("theme/fonts"), "myfont.woff", b"").unwrap();
    let config = Config::from_str("output.html.copy-fonts = false").unwrap();
    MDBook::load_with_config(p, config)
        .unwrap()
        .build()
        .unwrap();
    assert!(has_fonts_css(p));
    assert_eq!(
        actual_files(&p.join("book/fonts")),
        &["fonts.css", "myfont.woff"]
    );
}
