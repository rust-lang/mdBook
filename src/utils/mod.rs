#![allow(missing_docs)] // FIXME: Document this

pub mod fs;
mod string;
use errors::Error;
use regex::Regex;

use pulldown_cmark::{html, Event, Options, Parser, Tag, OPTION_ENABLE_FOOTNOTES,
                     OPTION_ENABLE_TABLES};

use std::borrow::Cow;

pub use self::string::{RangeArgument, take_lines};

/// Replaces multiple consecutive whitespace characters with a single space character.
pub fn collapse_whitespace<'a>(text: &'a str) -> Cow<'a, str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\s\s+").unwrap();
    }
    RE.replace_all(text, " ")
}

/// Convert the given string to a valid HTML element ID
pub fn normalize_id(content: &str) -> String {
    let mut ret = content
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
        .collect::<String>();
    // Ensure that the first character is [A-Za-z]
    if ret.chars().next().map_or(false, |c| !c.is_ascii_alphabetic()) {
        ret.insert(0, 'a');
    }
    ret
}

/// Generate an ID for use with anchors which is derived from a "normalised"
/// string.
pub fn id_from_content(content: &str) -> String {
    let mut content = content.to_string();

    // Skip any tags or html-encoded stuff
    const REPL_SUB: &[&str] = &["<em>",
                                "</em>",
                                "<code>",
                                "</code>",
                                "<strong>",
                                "</strong>",
                                "&lt;",
                                "&gt;",
                                "&amp;",
                                "&#39;",
                                "&quot;"];
    for sub in REPL_SUB {
        content = content.replace(sub, "");
    }

    // Remove spaces and hashes indicating a header
    let trimmed = content.trim().trim_left_matches('#').trim();

    normalize_id(trimmed)
}

/// Wrapper around the pulldown-cmark parser for rendering markdown to HTML.
pub fn render_markdown(text: &str, curly_quotes: bool) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);

    let mut opts = Options::empty();
    opts.insert(OPTION_ENABLE_TABLES);
    opts.insert(OPTION_ENABLE_FOOTNOTES);

    let p = Parser::new_ext(text, opts);
    let mut converter = EventQuoteConverter::new(curly_quotes);
    let events = p.map(clean_codeblock_headers)
                  .map(|event| converter.convert(event));

    html::push_html(&mut s, events);
    s
}

struct EventQuoteConverter {
    enabled: bool,
    convert_text: bool,
}

impl EventQuoteConverter {
    fn new(enabled: bool) -> Self {
        EventQuoteConverter {
            enabled: enabled,
            convert_text: true,
        }
    }

    fn convert<'a>(&mut self, event: Event<'a>) -> Event<'a> {
        if !self.enabled {
            return event;
        }

        match event {
            Event::Start(Tag::CodeBlock(_)) | Event::Start(Tag::Code) => {
                self.convert_text = false;
                event
            }
            Event::End(Tag::CodeBlock(_)) | Event::End(Tag::Code) => {
                self.convert_text = true;
                event
            }
            Event::Text(ref text) if self.convert_text => {
                Event::Text(Cow::from(convert_quotes_to_curly(text)))
            }
            _ => event,
        }
    }
}

fn clean_codeblock_headers(event: Event) -> Event {
    match event {
        Event::Start(Tag::CodeBlock(ref info)) => {
            let info: String = info.chars().filter(|ch| !ch.is_whitespace()).collect();

            Event::Start(Tag::CodeBlock(Cow::from(info)))
        }
        _ => event,
    }
}


fn convert_quotes_to_curly(original_text: &str) -> String {
    // We'll consider the start to be "whitespace".
    let mut preceded_by_whitespace = true;

    original_text.chars()
                 .map(|original_char| {
        let converted_char = match original_char {
            '\'' => {
                if preceded_by_whitespace {
                    '‘'
                } else {
                    '’'
                }
            }
            '"' => {
                if preceded_by_whitespace {
                    '“'
                } else {
                    '”'
                }
            }
            _ => original_char,
        };

        preceded_by_whitespace = original_char.is_whitespace();

        converted_char
    })
                 .collect()
}

/// Prints a "backtrace" of some `Error`.
pub fn log_backtrace(e: &Error) {
    error!("Error: {}", e);

    for cause in e.iter().skip(1) {
        error!("\tCaused By: {}", cause);
    }
}

#[cfg(test)]
mod tests {
    mod render_markdown {
        use super::super::render_markdown;

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

            let expected =
                r#"<pre><code class="language-rust,no_run,should_panic,property_3"></code></pre>
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

            let expected =
                r#"<pre><code class="language-rust,no_run,,,should_panic,,property_3"></code></pre>
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
    }

    mod html_munging {
        use super::super::{id_from_content, normalize_id};

        #[test]
        fn it_generates_anchors() {
            assert_eq!(id_from_content("## `--passes`: add more rustdoc passes"),
                    "a--passes-add-more-rustdoc-passes");
            assert_eq!(id_from_content("## Method-call expressions"),
                    "method-call-expressions");
        }

        #[test]
        fn it_normalizes_ids() {
            assert_eq!(normalize_id("`--passes`: add more rustdoc passes"),
                    "a--passes-add-more-rustdoc-passes");
            assert_eq!(normalize_id("Method-call 🐙 expressions \u{1f47c}"),
                    "method-call--expressions-");
            assert_eq!(normalize_id("_-_12345"), "a_-_12345");
            assert_eq!(normalize_id("12345"), "a12345");
            assert_eq!(normalize_id(""), "");
        }
    }

    mod convert_quotes_to_curly {
        use super::super::convert_quotes_to_curly;

        #[test]
        fn it_converts_single_quotes() {
            assert_eq!(convert_quotes_to_curly("'one', 'two'"),
                       "‘one’, ‘two’");
        }

        #[test]
        fn it_converts_double_quotes() {
            assert_eq!(convert_quotes_to_curly(r#""one", "two""#),
                       "“one”, “two”");
        }

        #[test]
        fn it_treats_tab_as_whitespace() {
            assert_eq!(convert_quotes_to_curly("\t'one'"), "\t‘one’");
        }
    }
}
