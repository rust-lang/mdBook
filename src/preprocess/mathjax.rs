//! Preprocessor that converts mathematical expression into MathJax.
//!
//! This preprocessor takes inline expressions wrapped in `$`-pairs and block
//! expressions wrapped in `$$`-pairs and transform them into a valid MathJax
//! expression that does not interfere with the markdown parser.

use errors::Result;
use regex::{CaptureMatches, Captures, Regex};

use super::{Preprocessor, PreprocessorContext};
use book::{Book, BookItem};

/// a preprocessor for expanding `$`- and `$$`-pairs into valid MathJax expressions.
pub struct MathJaxPreprocessor;

impl MathJaxPreprocessor {
    /// Create a `MathJaxPreprocessor`.
    pub fn new() -> Self {
        MathJaxPreprocessor
    }
}

impl Preprocessor for MathJaxPreprocessor {
    fn name(&self) -> &str {
        "mathjax"
    }

    fn run(&self, _ctx: &PreprocessorContext, book: &mut Book) -> Result<()> {
        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *section {
                let content = replace_all_mathematics(&chapter.content);
                chapter.content = content;
            }
        });

        Ok(())
    }
}

fn replace_all_mathematics(content: &str) -> String {
    let mut previous_end_index = 0;
    let mut replaced = String::new();

    for math in find_mathematics(content) {
        replaced.push_str(&content[previous_end_index..math.start_index]);
        replaced.push_str(&math.replacement());
        previous_end_index = math.end_index;
    }

    replaced.push_str(&content[previous_end_index..]);

    replaced
}

fn find_mathematics(content: &str) -> MathematicsIterator {
    lazy_static! {
        static ref REGEXP: Regex = Regex::new(r"(?x) # insignificant whitespace mode
                     # Mathematics is

                     # Block mathematics is
            (\$\$)   # a double dollar sign
            ([^$]+)  # followed by one or more things other than a dollar sign
            (\$\$)   # followed by a closing double dollar sign.

            |        # or

                     # Inline mathematics is
            (\$)     # a dollar sign
            ([^$]+)  # followed by one or more things other than a dollar sign
            (\$)     # followed by a closing dollar sign.

            |        # or

                     # Legacy inline mathematics
            (\\\\\() # An escaped opening bracket `\\(`
            ([^)]+)  # followed by one or more other things TODO provide correct regexp.
            (\\\\\)) # followed by a closing bracket `\\)`

            |        # or

                     # Legacy block mathematics
            (\\\\\[) # An escaped opening bracket `\\[`
            ([^$]+)  # followed by one ore more other things TODO provide correct regexp.
            (\\\\\]) # followed by a closing bracket `\\]`
        ").unwrap();
    }
    MathematicsIterator(REGEXP.captures_iter(content))
}

struct MathematicsIterator<'a>(CaptureMatches<'a, 'a>);

impl<'a> Iterator for MathematicsIterator<'a> {
    type Item = Mathematics<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for capture in &mut self.0 {
            if let mathematics @ Some(_) = Mathematics::from_capture(capture) {
                return mathematics;
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Mathematics<'a> {
    start_index: usize,
    end_index: usize,
    kind: Kind,
    text: &'a str,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Kind {
    Inline,
    Block,
    LegacyInline,
    LegacyBlock,
}

impl<'a> Mathematics<'a> {
    fn from_capture(captures: Captures<'a>) -> Option<Self> {
        let kind =
            captures.get(1).or(captures.get(4)).or(captures.get(7)).or(captures.get(10))
            .map(|delimiter|
                 match delimiter.as_str() {
                     "$$"   => Kind::Block,
                     "$"    => Kind::Inline,
                     r"\\[" => Kind::LegacyBlock,
                     _      => Kind::LegacyInline,
                 })
            .expect("captured mathematics should have opening delimiter at the provided indices");

        captures.get(0).map(|m| Mathematics {
            start_index: m.start(),
            end_index: m.end(),
            kind: kind,
            text: kind.text(m.as_str()),
        })
    }

    fn replacement(&self) -> String {
        let replacement: String = match self.kind {
            Kind::Block  | Kind::LegacyBlock  => {
                format!("<div class=\"math\">$${}$$</div>", self.text)
            },
            Kind::Inline | Kind::LegacyInline => {
                format!("<span class=\"inline math\">${}$</span>", self.text)
            },
        };
        replacement
    }
}

impl Kind {
    fn text<'a>(&self, delimited_text: &'a str) -> &'a str {
        let end = delimited_text.len();
        match *self {
            Kind::Block        => &delimited_text[2..end-2],
            Kind::Inline       => &delimited_text[1..end-1],
            Kind::LegacyBlock  => &delimited_text[3..end-3],
            Kind::LegacyInline => &delimited_text[3..end-3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_no_mathematics_in_regular_text() {
        let content = "Text without mathematics";

        assert_eq!(find_mathematics(content).count(), 0);
    }

    #[test]
    fn should_find_inline_mathematics() {
        let content = "Pythagorean theorem: $a^{2} + b^{2} = c^{2}$";

        let result = find_mathematics(content).collect::<Vec<_>>();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Mathematics {
            start_index: 21,
            end_index: 44,
            kind: Kind::Inline,
            text: "a^{2} + b^{2} = c^{2}",
        })
    }

    #[test]
    fn should_find_block_mathematics() {
        let content = "Euler's identity: $$e^{i\\pi} + 1 = 0$$";

        let result = find_mathematics(content).collect::<Vec<_>>();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Mathematics {
            start_index: 18,
            end_index: 38,
            kind: Kind::Block,
            text: "e^{i\\pi} + 1 = 0",
        })
    }

    #[test]
    fn should_find_legacy_inline_mathematics() {
        let content = r"Pythagorean theorem: \\(a^{2} + b^{2} = c^{2}\\)";

        let result = find_mathematics(content).collect::<Vec<_>>();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Mathematics {
            start_index: 21,
            end_index: 48,
            kind: Kind::LegacyInline,
            text: "a^{2} + b^{2} = c^{2}",
        })
    }

    #[test]
    fn should_find_legacy_block_mathematics() {
        let content = r"Euler's identity: \\[e^{i\pi} + 1 = 0\\]";

        let result = find_mathematics(content).collect::<Vec<_>>();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Mathematics {
            start_index: 18,
            end_index: 40,
            kind: Kind::LegacyBlock,
            text: "e^{i\\pi} + 1 = 0",
        })
    }

    #[test]
    fn should_replace_inline_mathematics() {
        let content = "Pythagorean theorem: $a^{2} + b^{2} = c^{2}$";

        let result = replace_all_mathematics(content);

        assert_eq!(result, "Pythagorean theorem: <span class=\"inline math\">$a^{2} + b^{2} = c^{2}$</span>")
    }

    #[test]
    fn should_replace_block_mathematics() {
        let content = "Euler's identity: $$e^{i\\pi} + 1 = 0$$";

        let result = replace_all_mathematics(content);

        assert_eq!(result, "Euler's identity: <div class=\"math\">$$e^{i\\pi} + 1 = 0$$</div>")
    }

}
