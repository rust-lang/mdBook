#![allow(missing_docs)] // FIXME: Document this

pub mod fs;
pub mod highlight;
mod string;
pub(crate) mod toml_ext;
use crate::errors::Error;
use regex::Regex;

use crate::config::{Playground, RustEdition};
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use syntect::html::ClassStyle;
use syntect::parsing::SyntaxReference;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

pub use self::string::{
    take_anchored_lines, take_lines, take_rustdoc_include_anchored_lines,
    take_rustdoc_include_lines,
};

/// Replaces multiple consecutive whitespace characters with a single space character.
pub fn collapse_whitespace(text: &str) -> Cow<'_, str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\s\s+").unwrap();
    }
    RE.replace_all(text, " ")
}

/// Convert the given string to a valid HTML element ID.
/// The only restriction is that the ID must not contain any ASCII whitespace.
pub fn normalize_id(content: &str) -> String {
    content
        .chars()
        .filter_map(|ch| {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
}

/// Generate an ID for use with anchors which is derived from a "normalised"
/// string.
// This function should be made private when the deprecation expires.
#[deprecated(since = "0.4.16", note = "use unique_id_from_content instead")]
pub fn id_from_content(content: &str) -> String {
    let mut content = content.to_string();

    // Skip any tags or html-encoded stuff
    lazy_static! {
        static ref HTML: Regex = Regex::new(r"(<.*?>)").unwrap();
    }
    content = HTML.replace_all(&content, "").into();
    const REPL_SUB: &[&str] = &["&lt;", "&gt;", "&amp;", "&#39;", "&quot;"];
    for sub in REPL_SUB {
        content = content.replace(sub, "");
    }

    // Remove spaces and hashes indicating a header
    let trimmed = content.trim().trim_start_matches('#').trim();
    normalize_id(trimmed)
}

/// Generate an ID for use with anchors which is derived from a "normalised"
/// string.
///
/// Each ID returned will be unique, if the same `id_counter` is provided on
/// each call.
pub fn unique_id_from_content(content: &str, id_counter: &mut HashMap<String, usize>) -> String {
    let id = {
        #[allow(deprecated)]
        id_from_content(content)
    };

    // If we have headers with the same normalized id, append an incrementing counter
    let id_count = id_counter.entry(id.clone()).or_insert(0);
    let unique_id = match *id_count {
        0 => id,
        id_count => format!("{}-{}", id, id_count),
    };
    *id_count += 1;
    unique_id
}

/// Fix links to the correct location.
///
/// This adjusts links, such as turning `.md` extensions to `.html`.
///
/// `path` is the path to the page being rendered relative to the root of the
/// book. This is used for the `print.html` page so that links on the print
/// page go to the original location. Normal page rendering sets `path` to
/// None. Ideally, print page links would link to anchors on the print page,
/// but that is very difficult.
fn adjust_links<'a>(event: Event<'a>, path: Option<&Path>) -> Event<'a> {
    lazy_static! {
        static ref SCHEME_LINK: Regex = Regex::new(r"^[a-z][a-z0-9+.-]*:").unwrap();
        static ref MD_LINK: Regex = Regex::new(r"(?P<link>.*)\.md(?P<anchor>#.*)?").unwrap();
    }

    fn fix<'a>(dest: CowStr<'a>, path: Option<&Path>) -> CowStr<'a> {
        if dest.starts_with('#') {
            // Fragment-only link.
            if let Some(path) = path {
                let mut base = path.display().to_string();
                if base.ends_with(".md") {
                    base.replace_range(base.len() - 3.., ".html");
                }
                return format!("{}{}", base, dest).into();
            } else {
                return dest;
            }
        }
        // Don't modify links with schemes like `https`.
        if !SCHEME_LINK.is_match(&dest) {
            // This is a relative link, adjust it as necessary.
            let mut fixed_link = String::new();
            if let Some(path) = path {
                let base = path
                    .parent()
                    .expect("path can't be empty")
                    .to_str()
                    .expect("utf-8 paths only");
                if !base.is_empty() {
                    write!(fixed_link, "{}/", base).unwrap();
                }
            }

            if let Some(caps) = MD_LINK.captures(&dest) {
                fixed_link.push_str(&caps["link"]);
                fixed_link.push_str(".html");
                if let Some(anchor) = caps.name("anchor") {
                    fixed_link.push_str(anchor.as_str());
                }
            } else {
                fixed_link.push_str(&dest);
            };
            return CowStr::from(fixed_link);
        }
        dest
    }

    fn fix_html<'a>(html: CowStr<'a>, path: Option<&Path>) -> CowStr<'a> {
        // This is a terrible hack, but should be reasonably reliable. Nobody
        // should ever parse a tag with a regex. However, there isn't anything
        // in Rust that I know of that is suitable for handling partial html
        // fragments like those generated by pulldown_cmark.
        //
        // There are dozens of HTML tags/attributes that contain paths, so
        // feel free to add more tags if desired; these are the only ones I
        // care about right now.
        lazy_static! {
            static ref HTML_LINK: Regex =
                Regex::new(r#"(<(?:a|img) [^>]*?(?:src|href)=")([^"]+?)""#).unwrap();
        }

        HTML_LINK
            .replace_all(&html, |caps: &regex::Captures<'_>| {
                let fixed = fix(caps[2].into(), path);
                format!("{}{}\"", &caps[1], fixed)
            })
            .into_owned()
            .into()
    }

    match event {
        Event::Start(Tag::Link(link_type, dest, title)) => {
            Event::Start(Tag::Link(link_type, fix(dest, path), title))
        }
        Event::Start(Tag::Image(link_type, dest, title)) => {
            Event::Start(Tag::Image(link_type, fix(dest, path), title))
        }
        Event::Html(html) => Event::Html(fix_html(html, path)),
        _ => event,
    }
}

/// Wrapper around the pulldown-cmark parser for rendering markdown to HTML.
pub fn render_markdown(
    text: &str,
    curly_quotes: bool,
    syntaxes: &SyntaxSet,
    playground_config: &Playground,
    default_edition: Option<RustEdition>,
) -> String {
    render_markdown_with_path(
        text,
        curly_quotes,
        None,
        syntaxes,
        playground_config,
        default_edition,
    )
}

pub fn new_cmark_parser(text: &str, curly_quotes: bool) -> Parser<'_, '_> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    if curly_quotes {
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    }
    Parser::new_ext(text, opts)
}

pub fn render_markdown_with_path(
    text: &str,
    curly_quotes: bool,
    path: Option<&Path>,
    syntaxes: &SyntaxSet,
    playground_config: &Playground,
    default_edition: Option<RustEdition>,
) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);
    let p = new_cmark_parser(text, curly_quotes);
    let mut highlighter = SyntaxHighlighter::new(playground_config, default_edition);
    let events = p
        .map(clean_codeblock_headers)
        .map(|event| adjust_links(event, path))
        .flat_map(|event| {
            let (a, b) = wrap_tables(event);
            a.into_iter().chain(b)
        })
        .map(|event| highlighter.highlight(syntaxes, event));

    html::push_html(&mut s, events);
    s
}

/// Wraps tables in a `.table-wrapper` class to apply overflow-x rules to.
fn wrap_tables(event: Event<'_>) -> (Option<Event<'_>>, Option<Event<'_>>) {
    match event {
        Event::Start(Tag::Table(_)) => (
            Some(Event::Html(r#"<div class="table-wrapper">"#.into())),
            Some(event),
        ),
        Event::End(Tag::Table(_)) => (Some(event), Some(Event::Html(r#"</div>"#.into()))),
        _ => (Some(event), None),
    }
}

struct SyntaxHighlighter<'a> {
    highlight: bool,
    is_rust: bool,
    is_playground: bool,
    is_editable: bool,
    syntax: Option<&'a SyntaxReference>,
    playground_config: &'a Playground,
    default_edition: Option<RustEdition>,
}

impl<'a> SyntaxHighlighter<'a> {
    fn new(playground_config: &'a Playground, default_edition: Option<RustEdition>) -> Self {
        SyntaxHighlighter {
            highlight: false,
            is_rust: false,
            is_playground: false,
            is_editable: false,
            syntax: None,
            playground_config,
            default_edition,
        }
    }

    fn highlight<'b>(&mut self, syntaxes: &'a SyntaxSet, event: Event<'b>) -> Event<'b> {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref info))) => {
                self.highlight = true;
                let mut classes: Vec<_> = info
                    .replace(",", " ")
                    .split(' ')
                    .map(String::from)
                    .filter(|x| !x.is_empty())
                    .collect();
                if let Some(name) = classes.first() {
                    // If we're given an empty name, or it is marked as
                    // plaintext, we shouldn't highlight it.
                    if name.is_empty()
                        || name.eq_ignore_ascii_case("plaintext")
                        || name.eq_ignore_ascii_case("text")
                        || name.eq_ignore_ascii_case("plain")
                        || name.eq_ignore_ascii_case("txt")
                    {
                        self.highlight = false;
                        return event;
                    }

                    self.syntax = syntaxes.find_syntax_by_token(name);
                    if self.syntax.is_none() {
                        self.highlight = false;
                        return event;
                    }
                    if let Some(syntax) = self.syntax {
                        if syntax.name == "Rust" {
                            self.is_rust = true;
                        }
                    }
                    if self.is_rust {
                        let ignore = classes.iter().find(|&x| x == "ignore").is_some();
                        let noplayground = classes.iter().find(|&x| x == "noplayground").is_some();
                        let noplaypen = classes.iter().find(|&x| x == "noplaypen").is_some();
                        let mdbook_runnable =
                            classes.iter().find(|&x| x == "mdbook-runnable").is_some();
                        // Enable playground
                        if (!ignore && !noplayground && !noplaypen) || mdbook_runnable {
                            self.is_editable = classes.iter().find(|&x| x == "editable").is_some();
                            let contains_e2015 =
                                classes.iter().find(|&x| x == "edition2015").is_some();
                            let contains_e2018 =
                                classes.iter().find(|&x| x == "edition2018").is_some();
                            let contains_e2021 =
                                classes.iter().find(|&x| x == "edition2021").is_some();
                            // if the user forced edition, we should not overwrite it
                            if !contains_e2015 && !contains_e2018 && !contains_e2021 {
                                match self.default_edition {
                                    Some(RustEdition::E2015) => {
                                        classes.push("edition2015".to_owned())
                                    }
                                    Some(RustEdition::E2018) => {
                                        classes.push("edition2018".to_owned())
                                    }
                                    Some(RustEdition::E2021) => {
                                        classes.push("edition2021".to_owned())
                                    }
                                    None => {}
                                }
                            }
                            self.is_playground = true;
                            return Event::Html(CowStr::from(format!(
                                r#"<pre class="playground"><code class="language-{}">"#,
                                classes.join(" ")
                            )));
                        }
                    }
                } else {
                    // We also don't perform auto-detection of languages, so we
                    // shouldn't highlight code blocks without lang tags.
                    self.highlight = false;
                }
                if classes.is_empty() {
                    Event::Html(CowStr::from("<pre><code>"))
                } else {
                    Event::Html(CowStr::from(format!(
                        r#"<pre><code class="language-{}">"#,
                        classes.join(" ")
                    )))
                }
            }
            Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                self.highlight = false;
                self.is_rust = false;
                self.syntax = None;
                self.is_playground = false;
                self.is_editable = false;
                Event::Html(CowStr::from("</code></pre>"))
            }
            Event::Text(ref code) if self.highlight => {
                let mut gen = highlight::HtmlGenerator::new(
                    self.syntax.unwrap(),
                    syntaxes,
                    ClassStyle::SpacedPrefixed { prefix: "syn-" },
                );
                let needs_wrapped = self.is_rust
                    && !(self.playground_config.editable && self.is_editable)
                    && !code.contains("fn main")
                    && !code.contains("quick_main!");
                if needs_wrapped {
                    gen.parse_line("# fn main() {\n", self.is_rust);
                }
                for line in LinesWithEndings::from(code) {
                    gen.parse_line(line, self.is_rust);
                }
                if needs_wrapped {
                    gen.parse_line("# }\n", self.is_rust);
                }
                Event::Html(CowStr::from(gen.finalize()))
            }
            _ => event,
        }
    }
}

fn clean_codeblock_headers(event: Event<'_>) -> Event<'_> {
    match event {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref info))) => {
            let info: String = info
                .chars()
                .map(|x| match x {
                    ' ' | '\t' => ',',
                    _ => x,
                })
                .filter(|ch| !ch.is_whitespace())
                .collect();

            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::from(info))))
        }
        _ => event,
    }
}

/// Prints a "backtrace" of some `Error`.
pub fn log_backtrace(e: &Error) {
    error!("Error: {}", e);

    for cause in e.chain().skip(1) {
        error!("\tCaused By: {}", cause);
    }
}

pub(crate) fn bracket_escape(mut s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    let needs_escape: &[char] = &['<', '>'];
    while let Some(next) = s.find(needs_escape) {
        escaped.push_str(&s[..next]);
        match s.as_bytes()[next] {
            b'<' => escaped.push_str("&lt;"),
            b'>' => escaped.push_str("&gt;"),
            _ => unreachable!(),
        }
        s = &s[next + 1..];
    }
    escaped.push_str(s);
    escaped
}

#[cfg(test)]
mod tests {
    use super::bracket_escape;

    mod render_markdown {
        use syntect::parsing::SyntaxSet;
        fn default_syntaxes() -> SyntaxSet {
            syntect::dumps::from_binary(crate::theme::SYNTAXES_BIN)
        }

        use super::super::render_markdown;
        use crate::config::{Playground, RustEdition};

        #[test]
        fn preserves_external_links() {
            assert_eq!(
                render_markdown(
                    "[example](https://www.rust-lang.org/)",
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                "<p><a href=\"https://www.rust-lang.org/\">example</a></p>\n"
            );
        }

        #[test]
        fn it_can_adjust_markdown_links() {
            assert_eq!(
                render_markdown(
                    "[example](example.md)",
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                "<p><a href=\"example.html\">example</a></p>\n"
            );
            assert_eq!(
                render_markdown(
                    "[example_anchor](example.md#anchor)",
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                "<p><a href=\"example.html#anchor\">example_anchor</a></p>\n"
            );

            // this anchor contains 'md' inside of it
            assert_eq!(
                render_markdown(
                    "[phantom data](foo.html#phantomdata)",
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
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
            assert_eq!(
                render_markdown(
                    src,
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                out
            );
        }

        #[test]
        fn it_can_keep_quotes_straight() {
            assert_eq!(
                render_markdown(
                    "'one'",
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                "<p>'one'</p>\n"
            );
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
            assert_eq!(
                render_markdown(
                    input,
                    true,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                expected
            );
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
<pre class="playground"><code class="language-rust"><span class="syn-source syn-rust"><span class="syn-meta syn-function syn-rust"><span class="syn-meta syn-function syn-rust"><span class="syn-storage syn-type syn-function syn-rust">fn</span> </span><span class="syn-entity syn-name syn-function syn-rust">main</span></span><span class="syn-meta syn-function syn-rust"><span class="syn-meta syn-function syn-parameters syn-rust"><span class="syn-punctuation syn-section syn-parameters syn-begin syn-rust">(</span></span><span class="syn-meta syn-function syn-rust"><span class="syn-meta syn-function syn-parameters syn-rust"><span class="syn-punctuation syn-section syn-parameters syn-end syn-rust">)</span></span></span></span><span class="syn-meta syn-function syn-rust"> </span><span class="syn-meta syn-function syn-rust"><span class="syn-meta syn-block syn-rust"><span class="syn-punctuation syn-section syn-block syn-begin syn-rust">{</span>
<span class="syn-comment syn-line syn-double-slash syn-rust"><span class="syn-punctuation syn-definition syn-comment syn-rust">//</span> code inside is unchanged
</span></span><span class="syn-meta syn-block syn-rust"><span class="syn-punctuation syn-section syn-block syn-end syn-rust">}</span></span></span>
</span></code></pre>
<p>more text with spaces</p>
"#;
            assert_eq!(
                render_markdown(
                    input,
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                expected
            );
            assert_eq!(
                render_markdown(
                    input,
                    true,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                expected
            );
        }

        #[test]
        fn rust_code_block_properties_are_passed_as_space_delimited_class() {
            let input = r#"
```rust,no_run,should_panic,property_3
```
"#;

            let expected = r#"<pre class="playground"><code class="language-rust no_run should_panic property_3"></code></pre>"#;
            assert_eq!(
                render_markdown(
                    input,
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                expected
            );
            assert_eq!(
                render_markdown(
                    input,
                    true,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                expected
            );
        }

        #[test]
        fn rust_code_block_properties_with_whitespace_are_passed_as_space_delimited_class() {
            let input = r#"
```rust,    no_run,,,should_panic , ,property_3
```
"#;

            let expected = r#"<pre class="playground"><code class="language-rust no_run should_panic property_3"></code></pre>"#;
            assert_eq!(
                render_markdown(
                    input,
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                expected
            );
            assert_eq!(
                render_markdown(
                    input,
                    true,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                expected
            );
        }

        #[test]
        fn rust_code_block_without_properties_has_proper_html_class() {
            let input = r#"
```rust
```
"#;

            let expected = r#"<pre class="playground"><code class="language-rust"></code></pre>"#;
            assert_eq!(
                render_markdown(
                    input,
                    false,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                expected
            );
            assert_eq!(
                render_markdown(
                    input,
                    true,
                    &default_syntaxes(),
                    &Playground::default(),
                    None,
                ),
                expected
            );
        }

        // These HTML strings get very, very long.
        // What I do is copy them out of here,
        // paste them into a document.write() call in the JavaScript console,
        // and then I can read the HTML in the DOM inspector to see if it looks right.
        #[test]
        fn add_playground() {
            let inputs = [
              ("```rust\nx()\n```",
               "<pre class=\"playground\"><code class=\"language-rust\"><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span>\n</span></span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-support syn-function syn-rust\">x</span><span class=\"syn-meta syn-group syn-rust\"><span class=\"syn-punctuation syn-section syn-group syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-group syn-rust\"><span class=\"syn-punctuation syn-section syn-group syn-end syn-rust\">)</span></span>\n</span></span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></span><span class=\"syn-source syn-rust\"></span></code></pre>"),
              ("```rust\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
              ("```rust editable\nlet s = \"foo\n # bar\n\";\n```",
               "<pre class=\"playground\"><code class=\"language-rust editable\"><span class=\"syn-source syn-rust\"><span class=\"syn-storage syn-type syn-rust\">let</span> s <span class=\"syn-keyword syn-operator syn-assignment syn-rust\">=</span> <span class=\"syn-string syn-quoted syn-double syn-rust\"><span class=\"syn-punctuation syn-definition syn-string syn-begin syn-rust\">&quot;</span>foo\n</span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\"> bar\n</span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\"><span class=\"syn-punctuation syn-definition syn-string syn-end syn-rust\">&quot;</span></span><span class=\"syn-punctuation syn-terminator syn-rust\">;</span>\n</span></code></pre>"),
              ("```rust editable\nlet s = \"foo\n ## bar\n\";\n```",
                "<pre class=\"playground\"><code class=\"language-rust editable\"><span class=\"syn-source syn-rust\"><span class=\"syn-storage syn-type syn-rust\">let</span> s <span class=\"syn-keyword syn-operator syn-assignment syn-rust\">=</span> <span class=\"syn-string syn-quoted syn-double syn-rust\"><span class=\"syn-punctuation syn-definition syn-string syn-begin syn-rust\">&quot;</span>foo\n</span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\"> # bar\n</span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\"><span class=\"syn-punctuation syn-definition syn-string syn-end syn-rust\">&quot;</span></span><span class=\"syn-punctuation syn-terminator syn-rust\">;</span>\n</span></code></pre>"),
              ("```rust editable\nlet s = \"foo\n # bar\n#\n\";\n```",
                "<pre class=\"playground\"><code class=\"language-rust editable\"><span class=\"syn-source syn-rust\"><span class=\"syn-storage syn-type syn-rust\">let</span> s <span class=\"syn-keyword syn-operator syn-assignment syn-rust\">=</span> <span class=\"syn-string syn-quoted syn-double syn-rust\"><span class=\"syn-punctuation syn-definition syn-string syn-begin syn-rust\">&quot;</span>foo\n</span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\"> bar\n</span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\"></span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\">\n</span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\"><span class=\"syn-punctuation syn-definition syn-string syn-end syn-rust\">&quot;</span></span><span class=\"syn-punctuation syn-terminator syn-rust\">;</span>\n</span></code></pre>"),
              ("```rust ignore\nlet s = \"foo\n # bar\n\";\n```",
                "<pre><code class=\"language-rust ignore\"><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span>\n</span></span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-storage syn-type syn-rust\">let</span> s <span class=\"syn-keyword syn-operator syn-assignment syn-rust\">=</span> <span class=\"syn-string syn-quoted syn-double syn-rust\"><span class=\"syn-punctuation syn-definition syn-string syn-begin syn-rust\">&quot;</span>foo\n</span></span></span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\"> bar\n</span></span></span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-string syn-quoted syn-double syn-rust\"><span class=\"syn-punctuation syn-definition syn-string syn-end syn-rust\">&quot;</span></span><span class=\"syn-punctuation syn-terminator syn-rust\">;</span>\n</span></span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></span><span class=\"syn-source syn-rust\"></span></code></pre>"),
              ("```rust editable\n#![no_std]\nlet s = \"foo\";\n #[some_attr]\n```",
                "<pre class=\"playground\"><code class=\"language-rust editable\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-annotation syn-rust\"><span class=\"syn-punctuation syn-definition syn-annotation syn-rust\">#!</span><span class=\"syn-punctuation syn-section syn-group syn-begin syn-rust\">[</span><span class=\"syn-variable syn-annotation syn-rust\">no_std</span><span class=\"syn-punctuation syn-section syn-group syn-end syn-rust\">]</span></span>\n<span class=\"syn-storage syn-type syn-rust\">let</span> s <span class=\"syn-keyword syn-operator syn-assignment syn-rust\">=</span> <span class=\"syn-string syn-quoted syn-double syn-rust\"><span class=\"syn-punctuation syn-definition syn-string syn-begin syn-rust\">&quot;</span>foo<span class=\"syn-punctuation syn-definition syn-string syn-end syn-rust\">&quot;</span></span><span class=\"syn-punctuation syn-terminator syn-rust\">;</span>\n <span class=\"syn-meta syn-annotation syn-rust\"><span class=\"syn-punctuation syn-definition syn-annotation syn-rust\">#</span><span class=\"syn-punctuation syn-section syn-group syn-begin syn-rust\">[</span><span class=\"syn-variable syn-annotation syn-rust\">some_attr</span><span class=\"syn-punctuation syn-section syn-group syn-end syn-rust\">]</span></span>\n</span></code></pre>"),
            ];
            for (src, should_be) in &inputs {
                let got = render_markdown(
                    src,
                    false,
                    &default_syntaxes(),
                    &Playground {
                        editable: true,
                        ..Playground::default()
                    },
                    None,
                );
                assert_eq!(&*got, *should_be);
            }
        }
        #[test]
        fn add_playground_edition2015() {
            let inputs = [
              ("```rust\nx()\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2015\"><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span>\n</span></span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-support syn-function syn-rust\">x</span><span class=\"syn-meta syn-group syn-rust\"><span class=\"syn-punctuation syn-section syn-group syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-group syn-rust\"><span class=\"syn-punctuation syn-section syn-group syn-end syn-rust\">)</span></span>\n</span></span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></span><span class=\"syn-source syn-rust\"></span></code></pre>"),
              ("```rust\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2015\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
              ("```rust edition2015\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2015\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
              ("```rust edition2018\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2018\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
            ];
            for (src, should_be) in &inputs {
                let got = render_markdown(
                    src,
                    false,
                    &default_syntaxes(),
                    &Playground {
                        editable: true,
                        ..Playground::default()
                    },
                    Some(RustEdition::E2015),
                );
                assert_eq!(&*got, *should_be);
            }
        }
        #[test]
        fn add_playground_edition2018() {
            let inputs = [
              ("```rust\nx()\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2018\"><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span>\n</span></span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-support syn-function syn-rust\">x</span><span class=\"syn-meta syn-group syn-rust\"><span class=\"syn-punctuation syn-section syn-group syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-group syn-rust\"><span class=\"syn-punctuation syn-section syn-group syn-end syn-rust\">)</span></span>\n</span></span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></span><span class=\"syn-source syn-rust\"></span></code></pre>"),
              ("```rust\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2018\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
              ("```rust edition2015\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2015\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
              ("```rust edition2018\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2018\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
            ];
            for (src, should_be) in &inputs {
                let got = render_markdown(
                    src,
                    false,
                    &default_syntaxes(),
                    &Playground {
                        editable: true,
                        ..Playground::default()
                    },
                    Some(RustEdition::E2018),
                );
                assert_eq!(&*got, *should_be);
            }
        }
        #[test]
        fn add_playground_edition2021() {
            let inputs = [
              ("```rust\nx()\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2021\"><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span>\n</span></span></span></span><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-support syn-function syn-rust\">x</span><span class=\"syn-meta syn-group syn-rust\"><span class=\"syn-punctuation syn-section syn-group syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-group syn-rust\"><span class=\"syn-punctuation syn-section syn-group syn-end syn-rust\">)</span></span>\n</span></span></span><span class=\"boring\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></span><span class=\"syn-source syn-rust\"></span></code></pre>"),
              ("```rust\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2021\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
              ("```rust edition2015\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2015\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
              ("```rust edition2018\nfn main() {}\n```",
               "<pre class=\"playground\"><code class=\"language-rust edition2018\"><span class=\"syn-source syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-storage syn-type syn-function syn-rust\">fn</span> </span><span class=\"syn-entity syn-name syn-function syn-rust\">main</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-begin syn-rust\">(</span></span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-function syn-parameters syn-rust\"><span class=\"syn-punctuation syn-section syn-parameters syn-end syn-rust\">)</span></span></span></span><span class=\"syn-meta syn-function syn-rust\"> </span><span class=\"syn-meta syn-function syn-rust\"><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-begin syn-rust\">{</span></span><span class=\"syn-meta syn-block syn-rust\"><span class=\"syn-punctuation syn-section syn-block syn-end syn-rust\">}</span></span></span>\n</span></code></pre>"),
            ];
            for (src, should_be) in &inputs {
                let got = render_markdown(
                    src,
                    false,
                    &default_syntaxes(),
                    &Playground {
                        editable: true,
                        ..Playground::default()
                    },
                    Some(RustEdition::E2021),
                );
                assert_eq!(&*got, *should_be);
            }
        }
        #[test]
        fn no_add_playground_to_other_languages() {
            let inputs = [
              ("```html,testhtml\n<p>\n```",
               "<pre><code class=\"language-html testhtml\"><span class=\"syn-text syn-html syn-basic\"><span class=\"syn-meta syn-tag syn-block syn-any syn-html\"><span class=\"syn-punctuation syn-definition syn-tag syn-begin syn-html\">&lt;</span><span class=\"syn-entity syn-name syn-tag syn-block syn-any syn-html\">p</span><span class=\"syn-punctuation syn-definition syn-tag syn-end syn-html\">&gt;</span></span>\n</span></code></pre>"),
              ("```js es7\nf()\n```",
               "<pre><code class=\"language-js es7\"><span class=\"syn-source syn-js\"><span class=\"syn-meta syn-function-call syn-without-arguments syn-js\"><span class=\"syn-variable syn-function syn-js\">f</span><span class=\"syn-meta syn-group syn-braces syn-round syn-function syn-arguments syn-js\">()</span></span>\n</span></code></pre>"),
            ];
            for (src, should_be) in &inputs {
                let got = render_markdown(
                    src,
                    false,
                    &default_syntaxes(),
                    &Playground {
                        editable: true,
                        ..Playground::default()
                    },
                    Some(RustEdition::E2021),
                );
                assert_eq!(&*got, *should_be);
            }
        }
    }

    #[allow(deprecated)]
    mod id_from_content {
        use super::super::id_from_content;

        #[test]
        fn it_generates_anchors() {
            assert_eq!(
                id_from_content("## Method-call expressions"),
                "method-call-expressions"
            );
            assert_eq!(id_from_content("## **Bold** title"), "bold-title");
            assert_eq!(id_from_content("## `Code` title"), "code-title");
            assert_eq!(
                id_from_content("## title <span dir=rtl>foo</span>"),
                "title-foo"
            );
        }

        #[test]
        fn it_generates_anchors_from_non_ascii_initial() {
            assert_eq!(
                id_from_content("## `--passes`: add more rustdoc passes"),
                "--passes-add-more-rustdoc-passes"
            );
            assert_eq!(
                id_from_content("## 中文標題 CJK title"),
                "中文標題-cjk-title"
            );
            assert_eq!(id_from_content("## Über"), "Über");
        }
    }

    mod html_munging {
        use super::super::{normalize_id, unique_id_from_content};

        #[test]
        fn it_normalizes_ids() {
            assert_eq!(
                normalize_id("`--passes`: add more rustdoc passes"),
                "--passes-add-more-rustdoc-passes"
            );
            assert_eq!(
                normalize_id("Method-call 🐙 expressions \u{1f47c}"),
                "method-call--expressions-"
            );
            assert_eq!(normalize_id("_-_12345"), "_-_12345");
            assert_eq!(normalize_id("12345"), "12345");
            assert_eq!(normalize_id("中文"), "中文");
            assert_eq!(normalize_id("にほんご"), "にほんご");
            assert_eq!(normalize_id("한국어"), "한국어");
            assert_eq!(normalize_id(""), "");
        }

        #[test]
        fn it_generates_unique_ids_from_content() {
            // Same id if not given shared state
            assert_eq!(
                unique_id_from_content("## 中文標題 CJK title", &mut Default::default()),
                "中文標題-cjk-title"
            );
            assert_eq!(
                unique_id_from_content("## 中文標題 CJK title", &mut Default::default()),
                "中文標題-cjk-title"
            );

            // Different id if given shared state
            let mut id_counter = Default::default();
            assert_eq!(unique_id_from_content("## Über", &mut id_counter), "Über");
            assert_eq!(
                unique_id_from_content("## 中文標題 CJK title", &mut id_counter),
                "中文標題-cjk-title"
            );
            assert_eq!(unique_id_from_content("## Über", &mut id_counter), "Über-1");
            assert_eq!(unique_id_from_content("## Über", &mut id_counter), "Über-2");
        }
    }

    #[test]
    fn escaped_brackets() {
        assert_eq!(bracket_escape(""), "");
        assert_eq!(bracket_escape("<"), "&lt;");
        assert_eq!(bracket_escape(">"), "&gt;");
        assert_eq!(bracket_escape("<>"), "&lt;&gt;");
        assert_eq!(bracket_escape("<test>"), "&lt;test&gt;");
        assert_eq!(bracket_escape("a<test>b"), "a&lt;test&gt;b");
    }
}
