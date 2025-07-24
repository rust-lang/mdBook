use super::render_markdown;
use super::*;

#[test]
fn escaped_special() {
    assert_eq!(special_escape(""), "");
    assert_eq!(special_escape("<"), "&lt;");
    assert_eq!(special_escape(">"), "&gt;");
    assert_eq!(special_escape("<>"), "&lt;&gt;");
    assert_eq!(special_escape("<test>"), "&lt;test&gt;");
    assert_eq!(special_escape("a<test>b"), "a&lt;test&gt;b");
    assert_eq!(special_escape("'"), "&#39;");
    assert_eq!(special_escape("\\"), "&#92;");
    assert_eq!(special_escape("&"), "&amp;");
}

#[test]
fn preserves_external_links() {
    assert_eq!(
        render_markdown("[example](https://www.rust-lang.org/)", false),
        "<p><a href=\"https://www.rust-lang.org/\">example</a></p>\n"
    );
}

#[test]
fn it_can_adjust_markdown_links() {
    assert_eq!(
        render_markdown("[example](example.md)", false),
        "<p><a href=\"example.html\">example</a></p>\n"
    );
    assert_eq!(
        render_markdown("[example_anchor](example.md#anchor)", false),
        "<p><a href=\"example.html#anchor\">example_anchor</a></p>\n"
    );

    // this anchor contains 'md' inside of it
    assert_eq!(
        render_markdown("[phantom data](foo.html#phantomdata)", false),
        "<p><a href=\"foo.html#phantomdata\">phantom data</a></p>\n"
    );
}

#[test]
fn it_can_wrap_tables() {
    let src = r#"
| Original        | Punycode        | Punycode + Encoding |
|-----------------|-----------------|---------------------|
| føø             | f-5gaa          | f_5gaa              |
"#;
    let out = r#"
<div class="table-wrapper"><table><thead><tr><th>Original</th><th>Punycode</th><th>Punycode + Encoding</th></tr></thead><tbody>
<tr><td>føø</td><td>f-5gaa</td><td>f_5gaa</td></tr>
</tbody></table>
</div>
"#.trim();
    assert_eq!(render_markdown(src, false), out);
}

#[test]
fn it_can_keep_quotes_straight() {
    assert_eq!(render_markdown("'one'", false), "<p>'one'</p>\n");
}

#[test]
fn it_can_make_quotes_curly_except_when_they_are_in_code() {
    let input = r#"
'one'
```
'two'
```
`'three'` 'four'"#;
    let expected = r#"<p>‘one’</p>
<pre><code>'two'
</code></pre>
<p><code>'three'</code> ‘four’</p>
"#;
    assert_eq!(render_markdown(input, true), expected);
}

#[test]
fn whitespace_outside_of_codeblock_header_is_preserved() {
    let input = r#"
some text with spaces
```rust
fn main() {
// code inside is unchanged
}
```
more text with spaces
"#;

    let expected = r#"<p>some text with spaces</p>
<pre><code class="language-rust">fn main() {
// code inside is unchanged
}
</code></pre>
<p>more text with spaces</p>
"#;
    assert_eq!(render_markdown(input, false), expected);
    assert_eq!(render_markdown(input, true), expected);
}

#[test]
fn rust_code_block_properties_are_passed_as_space_delimited_class() {
    let input = r#"
```rust,no_run,should_panic,property_3
```
"#;

    let expected = r#"<pre><code class="language-rust,no_run,should_panic,property_3"></code></pre>
"#;
    assert_eq!(render_markdown(input, false), expected);
    assert_eq!(render_markdown(input, true), expected);
}

#[test]
fn rust_code_block_properties_with_whitespace_are_passed_as_space_delimited_class() {
    let input = r#"
```rust,    no_run,,,should_panic , ,property_3
```
"#;

    let expected = r#"<pre><code class="language-rust,,,,,no_run,,,should_panic,,,,property_3"></code></pre>
"#;
    assert_eq!(render_markdown(input, false), expected);
    assert_eq!(render_markdown(input, true), expected);
}

#[test]
fn rust_code_block_without_properties_has_proper_html_class() {
    let input = r#"
```rust
```
"#;

    let expected = r#"<pre><code class="language-rust"></code></pre>
"#;
    assert_eq!(render_markdown(input, false), expected);
    assert_eq!(render_markdown(input, true), expected);

    let input = r#"
```rust
```
"#;
    assert_eq!(render_markdown(input, false), expected);
    assert_eq!(render_markdown(input, true), expected);
}
