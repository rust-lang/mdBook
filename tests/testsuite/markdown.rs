//! Tests for special markdown rendering.

use crate::prelude::*;

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
        .check_main_file("book/footnotes.html", str![[r##"
<h1 id="footnote-tests"><a class="header" href="#footnote-tests">Footnote tests</a></h1>
<p>Footnote example<sup class="footnote-reference" id="fr-1-1"><a href="#footnote-1">1</a></sup>, or with a word<sup class="footnote-reference" id="fr-word-1"><a href="#footnote-word">2</a></sup>.</p>
<p>There are multiple references to word<sup class="footnote-reference" id="fr-word-2"><a href="#footnote-word">2</a></sup>.</p>
<p>Footnote without a paragraph<sup class="footnote-reference" id="fr-para-1"><a href="#footnote-para">3</a></sup></p>
<p>Footnote with multiple paragraphs<sup class="footnote-reference" id="fr-multiple-1"><a href="#footnote-multiple">4</a></sup></p>
<p>Footnote name with wacky characters<sup class="footnote-reference" id="fr-&quot;wacky&quot;-1"><a href="#footnote-&quot;wacky&quot;">5</a></sup></p>
<p>Testing when referring to something earlier.<sup class="footnote-reference" id="fr-define-before-use-1"><a href="#footnote-define-before-use">6</a></sup></p>
<hr>
<ol class="footnote-definition"><li id="footnote-1">
<p>This is a footnote. <a href="#fr-1-1">↩</a> <a href="#fr-1-2">↩2</a></p>
</li>
<li id="footnote-word">
<p>A longer footnote.
With multiple lines. <a href="other.html">Link to other</a>.
With a reference inside.<sup class="footnote-reference" id="fr-1-2"><a href="#footnote-1">1</a></sup> <a href="#fr-word-1">↩</a> <a href="#fr-word-2">↩2</a></p>
</li>
<li id="footnote-para">
<ol>
<li>Item one
<ol>
<li>Sub-item</li>
</ol>
</li>
<li>Item two</li>
</ol>
 <a href="#fr-para-1">↩</a></li>
<li id="footnote-multiple">
<p>One</p>
<p>Two</p>
<p>Three <a href="#fr-multiple-1">↩</a></p>
</li>
<li id="footnote-&quot;wacky&quot;">
<p>Testing footnote id with special characters. <a href="#fr-&quot;wacky&quot;-1">↩</a></p>
</li>
<li id="footnote-define-before-use">
<p>This is defined before it is referred to. <a href="#fr-define-before-use-1">↩</a></p>
</li>
</ol>
"##]]);
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
