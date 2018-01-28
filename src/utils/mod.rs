#![allow(missing_docs)] // FIXME: Document this

pub mod fs;
mod link_filter;
mod string;
use errors::Error;

use pulldown_cmark::{html, Event, Options, Parser, Tag, OPTION_ENABLE_FOOTNOTES,
                     OPTION_ENABLE_TABLES};
use std::borrow::Cow;

pub use self::string::{RangeArgument, take_lines};
pub use self::link_filter::{LinkFilter, ChangeExtLinkFilter};

/// Wrapper around the pulldown-cmark parser for rendering markdown to HTML.
pub fn render_markdown(
    text: &str,
    link_filter: Option<&LinkFilter>,
    curly_quotes: bool,
) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);

    let mut opts = Options::empty();
    opts.insert(OPTION_ENABLE_TABLES);
    opts.insert(OPTION_ENABLE_FOOTNOTES);

    let p = Parser::new_ext(text, opts);

    let mut converter = EventQuoteConverter::new(curly_quotes);

    let events = p.map(clean_codeblock_headers)
                  .map(|event| converter.convert(event));

    let events: Box<Iterator<Item = Event>> = if let Some(filter) = link_filter {
        let mut link_filter_converter = LinkFilterConverter::new(filter);
        Box::new(events.map(move |event| link_filter_converter.convert(event)))
    } else {
        Box::new(events)
    };

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

struct LinkFilterConverter<'filter> {
    filter: &'filter LinkFilter,
}

impl<'filter> LinkFilterConverter<'filter> {
    fn new(filter: &'filter LinkFilter) -> Self {
        LinkFilterConverter {
            filter: filter,
        }
    }

    fn convert<'a>(&mut self, event: Event<'a>) -> Event<'a> {
        match event {
            Event::Start(Tag::Link(dest, title)) => {
                if let Some(translated) = self.filter.apply(&dest) {
                    return Event::Start(Tag::Link(Cow::Owned(translated), title));
                }

                Event::Start(Tag::Link(dest, title))
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
        use super::super::ChangeExtLinkFilter;
        use relative_path::RelativePath;

        #[test]
        fn it_can_keep_quotes_straight() {
            assert_eq!(render_markdown("'one'", None, false), "<p>'one'</p>\n");
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
            assert_eq!(render_markdown(input, None, true), expected);
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
            assert_eq!(render_markdown(input, None, false), expected);
            assert_eq!(render_markdown(input, None, true), expected);
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
            assert_eq!(render_markdown(input, None, false), expected);
            assert_eq!(render_markdown(input, None, true), expected);
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
            assert_eq!(render_markdown(input, None, false), expected);
            assert_eq!(render_markdown(input, None, true), expected);
        }

        #[test]
        fn rust_code_block_without_properties_has_proper_html_class() {
            let input = r#"
```rust
```
"#;

            let expected = r#"<pre><code class="language-rust"></code></pre>
"#;
            assert_eq!(render_markdown(input, None, false), expected);
            assert_eq!(render_markdown(input, None, true), expected);

            let input = r#"
```rust
```
"#;
            assert_eq!(render_markdown(input, None, false), expected);
            assert_eq!(render_markdown(input, None, true), expected);
        }

        #[test]
        fn test_link_filter() {
            let input = r#"
[foo](./bar.md)
[foo](./baz.md)
"#;

            let expected = "<p><a href=\"bar.html\">foo</a>\n<a href=\"./baz.md\">foo</a></p>\n";

            let bar = RelativePath::new("./bar.md");

            let filter = ChangeExtLinkFilter::new(
                RelativePath::new("."),
                |path| path == bar,
                "md",
                "html"
            );

            // only bar is a file.
            assert_eq!(render_markdown(input, Some(&filter), false), expected);
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
