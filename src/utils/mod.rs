pub mod fs;

use pulldown_cmark::{Parser, Event, Tag, html, Options, OPTION_ENABLE_TABLES, OPTION_ENABLE_FOOTNOTES};
use std::borrow::Cow;


///
///
/// Wrapper around the pulldown-cmark parser and renderer to render markdown

pub fn render_markdown(text: &str, curly_quotes: bool) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);

    let mut opts = Options::empty();
    opts.insert(OPTION_ENABLE_TABLES);
    opts.insert(OPTION_ENABLE_FOOTNOTES);

    let p = Parser::new_ext(text, opts);
    let mut converter = EventQuoteConverter::new(curly_quotes);
    let events = p.map(|event| converter.convert(event));
    html::push_html(&mut s, events);
    s
}

struct EventQuoteConverter {
    enabled: bool,
    convert_text: bool,
}

impl EventQuoteConverter {
    fn new(enabled: bool) -> Self {
        EventQuoteConverter { enabled: enabled, convert_text: true }
    }

    fn convert<'a>(&mut self, event: Event<'a>) -> Event<'a> {
        if !self.enabled {
            return event;
        }

        match event {
            Event::Start(Tag::CodeBlock(_)) |
            Event::Start(Tag::Code) => {
                self.convert_text = false;
                event
            },
            Event::End(Tag::CodeBlock(_)) |
            Event::End(Tag::Code) => {
                self.convert_text = true;
                event
            },
            Event::Text(ref text) if self.convert_text => Event::Text(Cow::from(convert_quotes_to_curly(text))),
            _ => event,
        }
    }
}

fn convert_quotes_to_curly(original_text: &str) -> String {
    // We'll consider the start to be "whitespace".
    let mut preceded_by_whitespace = true;

    original_text
        .chars()
        .map(|original_char| {
            let converted_char = match original_char {
                '\'' => if preceded_by_whitespace { '‘' } else { '’' },
                '"' => if preceded_by_whitespace { '“' } else { '”' },
                _ => original_char,
            };

            preceded_by_whitespace = original_char.is_whitespace();

            converted_char
        })
        .collect()
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
    }

    mod convert_quotes_to_curly {
        use super::super::convert_quotes_to_curly;

        #[test]
        fn it_converts_single_quotes() {
            assert_eq!(convert_quotes_to_curly("'one', 'two'"), "‘one’, ‘two’");
        }

        #[test]
        fn it_converts_double_quotes() {
            assert_eq!(convert_quotes_to_curly(r#""one", "two""#), "“one”, “two”");
        }

        #[test]
        fn it_treats_tab_as_whitespace() {
            assert_eq!(convert_quotes_to_curly("\t'one'"), "\t‘one’");
        }
    }
}
