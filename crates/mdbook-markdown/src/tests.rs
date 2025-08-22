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
    let options = HtmlRenderOptions::new(&Path::new(""));
    assert_eq!(
        render_markdown("[example](https://www.rust-lang.org/)", &options),
        "<p><a href=\"https://www.rust-lang.org/\">example</a></p>\n"
    );
}

#[test]
fn it_can_adjust_markdown_links() {
    let options = HtmlRenderOptions::new(&Path::new(""));
    assert_eq!(
        render_markdown("[example](example.md)", &options),
        "<p><a href=\"example.html\">example</a></p>\n"
    );
    assert_eq!(
        render_markdown("[example_anchor](example.md#anchor)", &options),
        "<p><a href=\"example.html#anchor\">example_anchor</a></p>\n"
    );

    // this anchor contains 'md' inside of it
    assert_eq!(
        render_markdown("[phantom data](foo.html#phantomdata)", &options),
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
    let options = HtmlRenderOptions::new(&Path::new(""));
    assert_eq!(render_markdown(src, &options), out);
}

#[test]
fn it_can_keep_quotes_straight() {
    let mut options = HtmlRenderOptions::new(&Path::new(""));
    options.markdown_options.smart_punctuation = false;
    assert_eq!(render_markdown("'one'", &options), "<p>'one'</p>\n");
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
    let options = HtmlRenderOptions::new(&Path::new(""));
    assert_eq!(render_markdown(input, &options), expected);
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
    let options = HtmlRenderOptions::new(&Path::new(""));
    assert_eq!(render_markdown(input, &options), expected);
}

#[test]
fn rust_code_block_properties_are_passed_as_space_delimited_class() {
    let input = r#"
```rust,no_run,should_panic,property_3
```
"#;

    let expected = r#"<pre><code class="language-rust,no_run,should_panic,property_3"></code></pre>
"#;
    let options = HtmlRenderOptions::new(&Path::new(""));
    assert_eq!(render_markdown(input, &options), expected);
}

#[test]
fn rust_code_block_properties_with_whitespace_are_passed_as_space_delimited_class() {
    let input = r#"
```rust,    no_run,,,should_panic , ,property_3
```
"#;

    let expected = r#"<pre><code class="language-rust,,,,,no_run,,,should_panic,,,,property_3"></code></pre>
"#;
    let options = HtmlRenderOptions::new(&Path::new(""));
    assert_eq!(render_markdown(input, &options), expected);
}

#[test]
fn rust_code_block_without_properties_has_proper_html_class() {
    let input = r#"
```rust
```
"#;

    let expected = r#"<pre><code class="language-rust"></code></pre>
"#;
    let options = HtmlRenderOptions::new(&Path::new(""));
    assert_eq!(render_markdown(input, &options), expected);

    let input = r#"
```rust
```
"#;
    let options = HtmlRenderOptions::new(&Path::new(""));
    assert_eq!(render_markdown(input, &options), expected);
}
