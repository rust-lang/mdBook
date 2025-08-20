//! Tests for special markdown rendering.

use crate::prelude::*;
use snapbox::file;

// Checks custom header id and classes.
#[test]
fn custom_header_attributes() {
    BookTest::from_dir("markdown/custom_header_attributes")
        .check_main_file("book/custom_header_attributes.html", str![[r##"
<h1 id="attrs"><a class="header" href="#attrs">Heading Attributes</a></h1>
<h2 id="heading-with-classes" class="class1 class2"><a class="header" href="#heading-with-classes">Heading with classes</a></h2>
<h2 id="both" class="class1 class2"><a class="header" href="#both">Heading with id and classes</a></h2>
"##]]);
}

// Test for a variety of footnote renderings.
#[test]
fn footnotes() {
    BookTest::from_dir("markdown/footnotes")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [WARN] (mdbook_markdown): footnote `multiple-definitions` in <unknown> defined multiple times - not updating to new definition
[TIMESTAMP] [WARN] (mdbook_markdown): footnote `unused` in `<unknown>` is defined but not referenced
[TIMESTAMP] [WARN] (mdbook_markdown): footnote `multiple-definitions` in footnotes.md defined multiple times - not updating to new definition
[TIMESTAMP] [WARN] (mdbook_markdown): footnote `unused` in `footnotes.md` is defined but not referenced
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

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
<div class="table-wrapper"><table><thead><tr><th>foo</th><th>bar</th></tr></thead><tbody>
<tr><td>baz</td><td>bim</td></tr>
<tr><td>Backslash in code</td><td><code>/</code></td></tr>
<tr><td>Double back in code</td><td><code>//</code></td></tr>
<tr><td>Pipe in code</td><td><code>|</code></td></tr>
<tr><td>Pipe in code2</td><td><code>test | inside</code></td></tr>
</tbody></table>
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
<li><input disabled="" type="checkbox" checked=""/>
Apples</li>
<li><input disabled="" type="checkbox" checked=""/>
Broccoli</li>
<li><input disabled="" type="checkbox"/>
Carrots</li>
</ul>
"##]],
    );
}

// Smart punctuation test.
#[test]
fn smart_punctuation() {
    BookTest::from_dir("markdown/smart_punctuation")
        // Default is off.
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
</ul>
"##]],
        )
        .run("build", |cmd| {
            cmd.env("MDBOOK_OUTPUT__HTML__SMART_PUNCTUATION", "true");
        })
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
</ul>
"##]],
        );
}
