//! Markdown processing used in mdBook.
//!
//! This crate provides functions for processing Markdown in the same way as
//! [mdBook](https://rust-lang.github.io/mdBook/). The [`pulldown_cmark`]
//! crate is used as the underlying parser. This crate re-exports
//! [`pulldown_cmark`] so that you can access its types.
//!
//! The parser in this library adds several modifications to the
//! [`pulldown_cmark`] event stream. For example, it adjusts some links,
//! modifies the behavior of footnotes, and adds various HTML wrappers.

use pulldown_cmark::{Options, Parser};

#[doc(inline)]
pub use pulldown_cmark;

/// Options for parsing markdown.
#[non_exhaustive]
pub struct MarkdownOptions {
    /// Enables smart punctuation.
    ///
    /// Converts quotes to curly quotes, `...` to `…`, `--` to en-dash, and
    /// `---` to em-dash.
    ///
    /// This is `true` by default.
    pub smart_punctuation: bool,
}

impl Default for MarkdownOptions {
    fn default() -> MarkdownOptions {
        MarkdownOptions {
            smart_punctuation: true,
        }
    }
}

/// Creates a new pulldown-cmark parser of the given text.
pub fn new_cmark_parser<'text>(text: &'text str, options: &MarkdownOptions) -> Parser<'text> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    if options.smart_punctuation {
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    }
    Parser::new_ext(text, opts)
}
