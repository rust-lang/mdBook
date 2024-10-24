use crate::errors::*;
use regex::{CaptureMatches, Captures, Regex};

use super::{Preprocessor, PreprocessorContext};
use crate::book::{Book, BookItem};
use log::{error, warn};
use once_cell::sync::Lazy;

const ESCAPE_CHAR: char = '\\';

/// A preprocessor for rendering in-text tooltip in a chapter.
///
/// - `[# title | content]` - Insert a specified content as hoverable element
///   on a webpage.
#[derive(Default)]
pub struct TooltipPreprocessor;

impl TooltipPreprocessor {
    pub(crate) const NAME: &'static str = "tooltips";

    /// Create a new `TooltipsPreprocessor`.
    pub fn new() -> Self {
        TooltipPreprocessor
    }
}

impl Preprocessor for TooltipPreprocessor {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut ch) = *section {
                ch.content = replace_all(&ch.content);
            }
        });

        Ok(book)
    }
}

fn replace_all(content: &str) -> String {
    // When replacing one thing in a string by something with a different length,
    // the indices after that will not correspond,
    // we therefore have to store the difference to correct this
    let mut previous_end_index = 0;
    let mut replaced = String::new();

    for tooltip in find_tooltips(content) {
        replaced.push_str(&content[previous_end_index..tooltip.start_index]);

        match tooltip.render() {
            Ok(new_content) => {
                replaced.push_str(&new_content);
                previous_end_index = tooltip.end_index;
            }
            Err(e) => {
                error!("Error updating \"{}\", {}", tooltip.tooltip_text, e);
                for cause in e.chain().skip(1) {
                    warn!("Caused By: {}", cause);
                }

                // This should make sure we include the raw `[# … | … ]` snippet
                // in the page content if there are any errors.
                previous_end_index = tooltip.start_index;
            }
        }
    }

    replaced.push_str(&content[previous_end_index..]);
    replaced
}

#[derive(PartialEq, Debug, Clone)]
enum TooltipType<'a> {
    Escaped,
    Inline(&'a str, &'a str),
}

#[derive(PartialEq, Debug, Clone)]
struct Tooltip<'a> {
    start_index: usize,
    end_index: usize,
    tooltip_type: TooltipType<'a>,
    tooltip_text: &'a str,
}

impl<'a> Tooltip<'a> {
    fn from_capture(cap: Captures<'a>) -> Option<Tooltip<'a>> {
        let tooltip_type = match (cap.get(0), cap.get(1), cap.get(2)) {
            (_, Some(title), Some(content)) => Some(TooltipType::Inline(
                title.as_str().trim(),
                content.as_str().trim(),
            )),
            (Some(mat), None, None) if mat.as_str().starts_with(ESCAPE_CHAR) => {
                Some(TooltipType::Escaped)
            }
            _ => None,
        };

        tooltip_type.and_then(|lnk_type| {
            cap.get(0).map(|mat| Tooltip {
                start_index: mat.start(),
                end_index: mat.end(),
                tooltip_type: lnk_type,
                tooltip_text: mat.as_str(),
            })
        })
    }

    fn render(&self) -> Result<String> {
        match self.tooltip_type {
            // omit the escape char
            TooltipType::Escaped => Ok(self.tooltip_text[1..].to_owned()),
            TooltipType::Inline(title, content) => {
                let tooltip_html = format!(
                    r#"<span class="text-tooltipped">{}<span class="text-tooltip-content">{}</span></span>"#,
                    title, content
                );

                Ok(tooltip_html)
            }
        }
    }
}

struct TooltipsIter<'a>(CaptureMatches<'a, 'a>);

impl<'a> Iterator for TooltipsIter<'a> {
    type Item = Tooltip<'a>;
    fn next(&mut self) -> Option<Tooltip<'a>> {
        for cap in &mut self.0 {
            if let Some(inc) = Tooltip::from_capture(cap) {
                return Some(inc);
            }
        }
        None
    }
}

fn find_tooltips(contents: &str) -> TooltipsIter<'_> {
    // lazily compute following regex
    // r"\\\[\#\s+([^|]+)\|([^\]]+)\]"
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?x)                  # insignificant whitespace mode
        \\\[\#\s+[^|]+\|[^\]]+\]    # match escaped tooltip
        |                           # or
        \[\#\s+                     # tooltip opening parens and whitespace
        ([^|]+)                     # \1 tooltip title
        \|                          # separating bar
        ([^\]]+)                    # \2 tooltip content
        \]                          # tooltip closing parens",
        )
        .unwrap()
    });

    TooltipsIter(RE.captures_iter(contents))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_all_escaped() {
        let text = r"[# random | not determined ]";
        let expected = r#"<span class="text-tooltipped">random<span class="text-tooltip-content">not determined</span></span>"#;

        let processed = replace_all(text);

        assert_eq!(processed, expected);
    }

    #[test]
    fn test_find_all_tooltip_types() {
        let text = "Some [# random | not determined ] text and \\[# escaped | tooltip ]";

        let res = find_tooltips(text).collect::<Vec<_>>();
        println!("\nOUTPUT: {res:?}\n");

        assert_eq!(res.len(), 2);
        assert_eq!(
            res[0],
            Tooltip {
                start_index: 5,
                end_index: 33,
                tooltip_type: TooltipType::Inline("random", "not determined"),
                tooltip_text: "[# random | not determined ]",
            }
        );
        assert_eq!(
            res[1],
            Tooltip {
                start_index: 43,
                end_index: 66,
                tooltip_type: TooltipType::Escaped,
                tooltip_text: "\\[# escaped | tooltip ]",
            }
        );
    }

    #[test]
    fn test_find_tooltips_no_tooltip() {
        let text = "Some random text without tooltip...";
        assert!(find_tooltips(text).collect::<Vec<_>>() == vec![]);
    }

    #[test]
    fn test_find_tooltips_partial_tooltip() {
        let text = "Some random text with [# one...";
        assert!(find_tooltips(text).collect::<Vec<_>>() == vec![]);
        let text = "Some random text with [# one...|";
        assert!(find_tooltips(text).collect::<Vec<_>>() == vec![]);
    }
}
