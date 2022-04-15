//! Tests for print page.

use crate::prelude::*;

// Tests relative links from the print page.
#[test]
fn relative_links() {
    BookTest::from_dir("print/relative_links")
        .check_main_file("book/print.html",
            str![[r##"
<div id="first-index"></div><h1 id="first-index-first-chapter"><a class="header" href="#first-index-first-chapter">First Chapter</a></h1>
<div style="break-before: page; page-break-before: always;"></div><div id="first-nested"></div><h1 id="first-nested-first-nested"><a class="header" href="#first-nested-first-nested">First Nested</a></h1>
<div style="break-before: page; page-break-before: always;"></div><div id="second-nested"></div><h1 id="second-nested-testing-relative-links-for-the-print-page"><a class="header" href="#second-nested-testing-relative-links-for-the-print-page">Testing relative links for the print page</a></h1>
<p>When we link to <a href="#first-nested">the first section</a>, it should work on
both the print page and the non-print page.</p>
<p>A <a href="#second-nested-some-section">fragment link</a> should work.</p>
<p>Link <a href="second/../../std/foo/bar.html">outside</a>.</p>
<p><img src="second/../images/picture.png" alt="Some image" /></p>
<p><a href="#first-markdown">HTML Link</a></p>
<img src="second/../images/picture.png" alt="raw html">
<h2 id="second-nested-some-section"><a class="header" href="#second-nested-some-section">Some section</a></h2>
"##]]);
}

// Checks that print.html is noindex.
#[test]
fn noindex() {
    let robots = r#"<meta name="robots" content="noindex">"#;
    BookTest::from_dir("print/noindex")
        .check_file_contains("book/print.html", robots)
        .check_file_doesnt_contain("book/index.html", robots);
}
