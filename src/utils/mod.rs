pub mod fs;

use pulldown_cmark::{html, Event, Options, Parser, Tag, OPTION_ENABLE_FOOTNOTES,
                     OPTION_ENABLE_TABLES};
use std::borrow::Cow;
use std::fmt::Write;
use regex::Regex;
use std::rc::Rc;

/// A heading together with the successive content until the next heading will
/// make up one `SearchDocument`. It represents some independently searchable part of the book.
#[derive(Default, Debug)]
pub struct SearchDocument {
    // Corresponding heading
    pub title : String,
    // Content: Flatted paragraphs, lists, code
    pub body : String,
    /// Needed information to generate a link to the corresponding title anchor
    /// First part is the `reference_base` that should be the same for all documents that
    /// came from the same `.md` file. The second part is derived from the heading of the search
    /// document.
    pub sref : (Rc<String>, Option<String>),
    // Breadcrumbs like ["Main Chapter Title", "Sub Chapter Title", "H1 Heading"]
    // as a human understandable path to the search document.
    pub breadcrumbs : Vec<Rc<String>>,
}

impl SearchDocument {
    fn new(sref0 : &Rc<String>, bcs : &Vec<Rc<String>>) -> SearchDocument {
        SearchDocument {
            title : "".to_owned(),
            body : "".to_owned(),
            sref : (sref0.clone(), None),
            breadcrumbs : bcs.clone()
        }
    }

    fn has_content(&self) -> bool {
        self.title.len() > 0
    }

    fn add(&mut self, text : &str, to_title : bool) {
        if to_title {
            self.title.write_str(&text).unwrap();
        } else {
            self.body.write_str(&text).unwrap();
            self.body.write_str(&" ").unwrap();
        }
    }
}

/// Renders markdown into flat unformatted text for usage in the search index.
/// Refer to the struct `SearchDocument`.
///
/// The field `sref` in the `SearchDocument` struct becomes
///    `(reference_base, Some(heading_to_sref("The Section Heading")))`
pub fn render_markdown_into_searchindex<F>(
    search_documents: &mut Vec<SearchDocument>,
    text: &str,
    reference_base: &str,
    breadcrumbs : &Vec<Rc<String>>,
    heading_to_sref : F)
    where F : Fn(&str) -> String {

    let mut opts = Options::empty();
    opts.insert(OPTION_ENABLE_TABLES);
    opts.insert(OPTION_ENABLE_FOOTNOTES);
    let p = Parser::new_ext(text, opts);

    let reference_base = Rc::new(reference_base.to_owned());
    let mut current = SearchDocument::new(&reference_base, breadcrumbs);
    let mut in_header = false;

    for event in p {
        match event {
            Event::Start(Tag::Header(i)) if i <= 3 => {
                if current.has_content() {
                    search_documents.push(current);
                }
                current = SearchDocument::new(&reference_base, breadcrumbs);
                in_header = true;
            }
            Event::End(Tag::Header(_)) => {
                // Possible extension: Use h1,h2,h3 as hierarchy for the breadcrumbs
                current.breadcrumbs.push(Rc::new(current.title.clone()));
                current.sref.1 = Some(heading_to_sref(&current.title));
                in_header = false;
            }
            Event::Start(_) | Event::End(_) => {}
            Event::Text(text) => {
                current.add(&text, in_header);
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                current.body.write_str(&trim_html_tags(&html)).unwrap();
            }
            Event::FootnoteReference(_) => {}
            Event::SoftBreak | Event::HardBreak => {}
        }
    }
    search_documents.push(current);
}

fn trim_html_tags<'a>(text : &'a str) -> Cow<'a, str> {
    let regex = Regex::new(r"<[^>]*?>").unwrap();
    regex.replace_all(text, "")
}

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
