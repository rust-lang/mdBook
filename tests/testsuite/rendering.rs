//! Tests for HTML rendering.
//!
//! Note that markdown-specific rendering tests are in the `markdown` module.

use crate::prelude::*;

// Checks that edit-url-template works.
#[test]
fn edit_url_template() {
    BookTest::from_dir("rendering/edit_url_template").check_file_contains(
        "book/index.html",
        "<a href=\"https://github.com/rust-lang/mdBook/edit/master/guide/src/README.md\" \
         title=\"Suggest an edit\" aria-label=\"Suggest an edit\" rel=\"edit\">",
    );
}

// Checks that an alternate `src` setting works with the edit url template.
#[test]
fn edit_url_template_explicit_src() {
    BookTest::from_dir("rendering/edit_url_template_explicit_src").check_file_contains(
        "book/index.html",
        "<a href=\"https://github.com/rust-lang/mdBook/edit/master/guide/src2/README.md\" \
         title=\"Suggest an edit\" aria-label=\"Suggest an edit\" rel=\"edit\">",
    );
}

// Checks that index.html is generated correctly, even when the first few
// chapters are drafts.
#[test]
fn first_chapter_is_copied_as_index_even_if_not_first_elem() {
    BookTest::from_dir("rendering/first_chapter_is_copied_as_index_even_if_not_first_elem")
        // These two files should be equal.
        .check_main_file(
            "book/chapter_1.html",
            str![[
                r##"<h1 id="chapter-1"><a class="header" href="#chapter-1">Chapter 1</a></h1>"##
            ]],
        )
        .check_main_file(
            "book/index.html",
            str![[
                r##"<h1 id="chapter-1"><a class="header" href="#chapter-1">Chapter 1</a></h1>"##
            ]],
        );
}

// Fontawesome `<i>` tag support.
#[test]
fn fontawesome() {
    BookTest::from_dir("rendering/fontawesome")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 WARN failed to find Font Awesome icon for icon `does-not-exist` with type `regular` in `fa.md`: Invalid Font Awesome icon name: visit https://fontawesome.com/icons?d=gallery&m=free to see valid names
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_all_main_files();
}

// Verifies that an invalid `git-repository-icon` in book.toml produces a
// helpful error message with the icon name, type, and a link to FontAwesome.
#[test]
fn fontawesome_error_message() {
    BookTest::from_dir("rendering/fontawesome_error")
        .run("build", |cmd| {
            cmd.expect_failure();
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
ERROR Rendering failed
[TAB]Caused by: Error rendering "index" line [..], col [..]: Unknown Font Awesome icon `github` for type `regular`. Hint: check the icon name and prefix (fas (solid), fab (brands), or far (regular)) at https://fontawesome.com/v6/search?m=free
[TAB]Caused by: Unknown Font Awesome icon `github` for type `regular`. Hint: check the icon name and prefix (fas (solid), fab (brands), or far (regular)) at https://fontawesome.com/v6/search?m=free

"#]]);
        });
}

// Tests the rendering when setting the default rust edition.
#[test]
fn default_rust_edition() {
    BookTest::from_dir("rendering/default_rust_edition").check_all_main_files();
}

// Tests the rendering for editable code blocks.
#[test]
fn editable_rust_block() {
    BookTest::from_dir("rendering/editable_rust_block").check_all_main_files();
}

// Tests for custom hide lines.
#[test]
fn hidelines() {
    BookTest::from_dir("rendering/hidelines").check_all_main_files();
}

// Tests for code blocks of basic rust code.
#[test]
fn language_rust_playground() {
    fn expect(input: &str, info: &str, expected: impl snapbox::IntoData) {
        BookTest::init(|_| {})
            .change_file("book.toml", "output.html.playground.editable = true")
            .change_file("src/chapter_1.md", &format!("```rust {info}\n{input}\n```"))
            .check_main_file("book/chapter_1.html", expected);
    }
    // No-main should be wrapped in `fn main` boring lines.
    expect(
        "x()",
        "",
        str![[r#"
<pre class="playground"><code class="language-rust"><span class="boring">#![allow(unused)]
</span><span class="boring">fn main() {
</span>x()
<span class="boring">}</span></code></pre>
"#]],
    );
    // `fn main` should not be wrapped, not boring.
    expect(
        "fn main() {}",
        "",
        str![[r#"<pre class="playground"><code class="language-rust">fn main() {}</code></pre>"#]],
    );
    // Lines starting with `#` are boring.
    expect(
        "let s = \"foo\n # bar\n\";",
        "editable",
        str![[r#"
<pre class="playground"><code class="language-rust editable">let s = "foo
<span class="boring"> bar
</span>";</code></pre>
"#]],
    );
    // `##` is not boring and is used as an escape.
    expect(
        "let s = \"foo\n ## bar\n\";",
        "editable",
        str![[r#"
<pre class="playground"><code class="language-rust editable">let s = "foo
 # bar
";</code></pre>
"#]],
    );
    // `#` on a line by itself is boring.
    expect(
        "let s = \"foo\n # bar\n#\n\";",
        "editable",
        str![[r#"
<pre class="playground"><code class="language-rust editable">let s = "foo
<span class="boring"> bar
</span><span class="boring">
</span>";</code></pre>
"#]],
    );
    // `#` must be followed by a space to be boring.
    expect(
        "#x;",
        "",
        str![[r#"
<pre class="playground"><code class="language-rust"><span class="boring">#![allow(unused)]
</span><span class="boring">fn main() {
</span>#x;
<span class="boring">}</span></code></pre>
"#]],
    );

    // Other classes like "ignore" should not change things, and the class is
    // included in the code tag.
    expect(
        "let s = \"foo\n # bar\n\";",
        "ignore",
        str![[r#"
<pre><code class="language-rust ignore">let s = "foo
<span class="boring"> bar
</span>";</code></pre>
"#]],
    );
    // Inner attributes and normal attributes are not boring.
    expect(
        "#![no_std]\nlet s = \"foo\";\n #[some_attr]",
        "editable",
        str![[r#"
<pre class="playground"><code class="language-rust editable">#![no_std]
let s = "foo";
 #[some_attr]</code></pre>
"#]],
    );
}

// Rust code block in a list.
#[test]
fn code_block_in_list() {
    BookTest::init(|_| {})
        .change_file(
            "src/chapter_1.md",
            r#"- inside list

  ```rust
  fn foo() {
    let x = 1;
  }
  ```
"#,
        )
        .check_main_file(
            "book/chapter_1.html",
            str![[r#"
<ul>
<li>
<p>inside list</p>
<pre class="playground"><code class="language-rust"><span class="boring">#![allow(unused)]
</span><span class="boring">fn main() {
</span>fn foo() {
  let x = 1;
}
<span class="boring">}</span></code></pre>
</li>
</ul>
"#]],
        );
}

// Checks the rendering of links added to headers.
#[test]
fn header_links() {
    BookTest::from_dir("rendering/header_links").check_all_main_files();
}

// A corrupted HTML end tag.
#[test]
fn busted_end_tag() {
    BookTest::init(|_| {})
        .change_file("src/chapter_1.md", "<div>x<span>foo</span/>y</div>")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 WARN html parse error in `chapter_1.md`: Self-closing end tag
Html text was:
<div>x<span>foo</span/>y</div>
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_main_file("book/chapter_1.html", str!["<div>x<span>foo</span>y</div>"]);
}

// Various html blocks.
#[test]
fn html_blocks() {
    BookTest::from_dir("rendering/html_blocks").check_all_main_files();
}

// Test for a fenced code block that is also indented.
#[test]
fn code_block_fenced_with_indent() {
    BookTest::from_dir("rendering/code_blocks_fenced_with_indent").check_all_main_files();
}

// Unclosed HTML tags.
//
// Note that the HTML parsing algorithm is much more complicated than what
// this is checking.
#[test]
fn unclosed_html_tags() {
    BookTest::init(|_| {})
        .change_file("src/chapter_1.md", "<div>x<span>foo<i>xyz")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 WARN unclosed HTML tag `<i>` found in `chapter_1.md`
 WARN unclosed HTML tag `<span>` found in `chapter_1.md`
 WARN unclosed HTML tag `<div>` found in `chapter_1.md`
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_main_file(
            "book/chapter_1.html",
            str!["<div>x<span>foo<i>xyz</i></span></div>"],
        );
}

// Test for HTML tags out of sync.
#[test]
fn unbalanced_html_tags() {
    BookTest::init(|_| {})
        .change_file("src/chapter_1.md", "<div>x<span>foo</div></span>")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 WARN unexpected HTML end tag `</div>` found in `chapter_1.md`
Check that the HTML tags are properly balanced.
 WARN unclosed HTML tag `<div>` found in `chapter_1.md`
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_main_file("book/chapter_1.html", str!["<div>x<span>foo</span></div>"]);
}

// Test for bug with unbalanced HTML handling in the heading.
#[test]
fn heading_with_unbalanced_html() {
    BookTest::init(|_| {})
        .change_file("src/chapter_1.md", "### Option<T>")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 WARN unclosed HTML tag `<t>` found in `chapter_1.md` while exiting Heading(H3)
HTML tags must be closed before exiting a markdown element.
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_main_file(
            "book/chapter_1.html",
            str![[r##"<h3 id="option"><a class="header" href="#option">Option<t></t></a></h3>"##]],
        );
}
