//! HTML rendering support.
//!
//! This module's primary entry point is [`render_markdown`] which will take
//! markdown text and render it to HTML. In summary, the general procedure of
//! that function is:
//!
//! 1. Use [`pulldown_cmark`] to parse the markdown and generate events.
//! 2. [`tree`] converts those events to a tree data structure.
//!      1. Parse HTML inside the markdown using [`tokenizer`].
//!      2. Apply various transformations to the tree data structure, such as adding header links.
//! 3. Serialize the tree to HTML in [`serialize()`].

use ego_tree::Tree;
use mdbook_core::book::{Book, Chapter};
use mdbook_core::config::{HtmlConfig, RustEdition};
use mdbook_markdown::{MarkdownOptions, new_cmark_parser};
use std::path::{Path, PathBuf};

mod admonitions;
mod hide_lines;
mod print;
mod serialize;
#[cfg(test)]
mod tests;
mod tokenizer;
mod tree;

pub(crate) use hide_lines::{hide_lines, wrap_rust_main};
pub(crate) use print::render_print_page;
pub(crate) use serialize::serialize;
pub(crate) use tree::{Element, Node};

/// Options for converting a single chapter's markdown to HTML.
pub(crate) struct HtmlRenderOptions<'a> {
    /// Options for parsing markdown.
    pub markdown_options: MarkdownOptions,
    /// The chapter's location, relative to the `SUMMARY.md` file.
    pub path: &'a Path,
    /// The default Rust edition, used to set the proper class on the code blocks.
    pub edition: Option<RustEdition>,
    /// The [`HtmlConfig`], whose options affect how the HTML is generated.
    pub config: &'a HtmlConfig,
}

impl<'a> HtmlRenderOptions<'a> {
    /// Creates a new [`HtmlRenderOptions`].
    pub(crate) fn new(
        path: &'a Path,
        config: &'a HtmlConfig,
        edition: Option<RustEdition>,
    ) -> HtmlRenderOptions<'a> {
        let mut markdown_options = MarkdownOptions::default();
        markdown_options.smart_punctuation = config.smart_punctuation;
        markdown_options.definition_lists = config.definition_lists;
        markdown_options.admonitions = config.admonitions;
        HtmlRenderOptions {
            markdown_options,
            path,
            edition,
            config,
        }
    }
}

/// Renders markdown to HTML.
pub(crate) fn render_markdown(text: &str, options: &HtmlRenderOptions<'_>) -> String {
    let tree = build_tree(text, options);
    let mut output = String::new();
    serialize::serialize(&tree, &mut output);
    output
}

/// Renders markdown to a [`Tree`].
fn build_tree(text: &str, options: &HtmlRenderOptions<'_>) -> Tree<Node> {
    let events = new_cmark_parser(text, &options.markdown_options);
    tree::MarkdownTreeBuilder::build(options, events)
}

/// The parsed chapter, and some information about the chapter.
pub(crate) struct ChapterTree<'book> {
    pub(crate) chapter: &'book Chapter,
    /// The path to the chapter relative to the root with the `.html` extension.
    pub(crate) html_path: PathBuf,
    /// The chapter tree.
    pub(crate) tree: Tree<Node>,
}

/// Creates all of the [`ChapterTree`]s for the book.
pub(crate) fn build_trees<'book>(
    book: &'book Book,
    html_config: &HtmlConfig,
    edition: Option<RustEdition>,
) -> Vec<ChapterTree<'book>> {
    book.chapters()
        .map(|ch| {
            let path = ch.path.as_ref().unwrap();
            let html_path = ch.path.as_ref().unwrap().with_extension("html");
            let options = HtmlRenderOptions::new(path, html_config, edition);
            let tree = build_tree(&ch.content, &options);

            ChapterTree {
                chapter: ch,
                html_path,
                tree,
            }
        })
        .collect()
}
