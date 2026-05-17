//! Tests for special markdown rendering.

use crate::prelude::*;
use snapbox::file;

// Checks custom header id and classes.
#[test]
fn custom_header_attributes() {
    BookTest::from_dir("markdown/custom_header_attributes")
        .check_main_file("book/custom_header_attributes.html", str![[r##"
<h1 id="attrs"><a class="header" href="#attrs">Heading Attributes</a></h1>
<h2 class="class1 class2" id="heading-with-classes"><a class="header" href="#heading-with-classes">Heading with classes</a></h2>
<h2 id="both" class="class1 class2"><a class="header" href="#both">Heading with id and classes</a></h2>
<h2 myattr="" otherattr="value" id="myh3" class="myclass1 myclass2"><a class="header" href="#myh3">Heading with attribute</a></h2>
"##]]);
}

// Test for a variety of footnote renderings.
#[test]
fn footnotes() {
    BookTest::from_dir("markdown/footnotes")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 WARN footnote `multiple-definitions` in footnotes.md defined multiple times - not updating to new definition
 WARN footnote `unused` in `footnotes.md` is defined but not referenced
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_main_file(
            "book/footnotes.html",
            file!["markdown/footnotes/expected/footnotes.html"],
        );
}

// Basic table test.
#[test]
fn tables() {
    BookTest::from_dir("markdown/tables").check_main_file(
        "book/tables.html",
        str![[r##"
<h1 id="tables"><a class="header" href="#tables">Tables</a></h1>
<div class="table-wrapper">
<table>
<thead>
<tr><th>foo</th><th>bar</th></tr>
</thead>
<tbody>
<tr><td>baz</td><td>bim</td></tr>
<tr><td>Backslash in code</td><td><code>/</code></td></tr>
<tr><td>Double back in code</td><td><code>//</code></td></tr>
<tr><td>Pipe in code</td><td><code>|</code></td></tr>
<tr><td>Pipe in code2</td><td><code>test | inside</code></td></tr>
</tbody>
</table>
</div>
<div class="table-wrapper">
<table>
<thead>
<tr><th>Neither</th><th style="text-align: left">Left</th><th style="text-align: center">Center</th><th style="text-align: right">Right</th></tr>
</thead>
<tbody>
<tr><td>one</td><td style="text-align: left">two</td><td style="text-align: center">three</td><td style="text-align: right">four</td></tr>
</tbody>
</table>
</div>
"##]],
    );
}

// Strikethrough test.
#[test]
fn strikethrough() {
    BookTest::from_dir("markdown/strikethrough").check_main_file(
        "book/strikethrough.html",
        str![[r##"
<h1 id="strikethrough"><a class="header" href="#strikethrough">Strikethrough</a></h1>
<p><del>strikethrough example</del></p>
"##]],
    );
}

// Tasklist test.
#[test]
fn tasklists() {
    BookTest::from_dir("markdown/tasklists").check_main_file(
        "book/tasklists.html",
        str![[r##"
<h2 id="tasklisks"><a class="header" href="#tasklisks">Tasklisks</a></h2>
<ul>
<li><input disabled="" type="checkbox" checked=""> Apples</li>
<li><input disabled="" type="checkbox" checked=""> Broccoli</li>
<li><input disabled="" type="checkbox"> Carrots</li>
</ul>
"##]],
    );
}

// Smart punctuation test.
#[test]
fn smart_punctuation() {
    BookTest::from_dir("markdown/smart_punctuation")
        // Default is on.
        .check_main_file(
            "book/smart_punctuation.html",
            str![[r##"
<h1 id="smart-punctuation"><a class="header" href="#smart-punctuation">Smart Punctuation</a></h1>
<ul>
<li>En dash: –</li>
<li>Em dash: —</li>
<li>Ellipsis: …</li>
<li>Double quote: “quote”</li>
<li>Single quote: ‘quote’</li>
<li>Quote in <code>"code"</code></li>
</ul>
<pre><code>"quoted"
</code></pre>
"##]],
        )
        .run("build", |cmd| {
            cmd.env("MDBOOK_OUTPUT__HTML__SMART_PUNCTUATION", "false");
        })
        .check_main_file(
            "book/smart_punctuation.html",
            str![[r##"
<h1 id="smart-punctuation"><a class="header" href="#smart-punctuation">Smart Punctuation</a></h1>
<ul>
<li>En dash: --</li>
<li>Em dash: ---</li>
<li>Ellipsis: ...</li>
<li>Double quote: "quote"</li>
<li>Single quote: 'quote'</li>
<li>Quote in <code>"code"</code></li>
</ul>
<pre><code>"quoted"
</code></pre>
"##]],
        );
}

// Basic markdown syntax.
// This doesn't try to cover the commonmark test suite, but maybe it could some day?
#[test]
fn basic_markdown() {
    BookTest::from_dir("markdown/basic_markdown").check_all_main_files();
}

#[test]
fn definition_lists() {
    BookTest::from_dir("markdown/definition_lists")
        .check_all_main_files()
        .run("build", |cmd| {
            cmd.env("MDBOOK_OUTPUT__HTML__DEFINITION_LISTS", "false");
        })
        .check_main_file(
            "book/definition_lists.html",
            file!["markdown/definition_lists/expected_disabled/definition_lists.html"],
        )
        .check_main_file(
            "book/html_definition_lists.html",
            file!["markdown/definition_lists/expected_disabled/html_definition_lists.html"],
        );
}

#[test]
fn admonitions() {
    BookTest::from_dir("markdown/admonitions")
        .check_all_main_files()
        .run("build", |cmd| {
            cmd.env("MDBOOK_OUTPUT__HTML__ADMONITIONS", "false");
        })
        .check_main_file(
            "book/admonitions.html",
            file!["markdown/admonitions/expected_disabled/admonitions.html"],
        );
}
