use anyhow::Result;
use mdbook_core::book::{Book, BookItem};
use mdbook_core::static_regex;
use mdbook_markdown::pulldown_cmark::{Event, LinkType, Tag};
use mdbook_markdown::{MarkdownOptions, new_cmark_parser};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use std::path::{Path, PathBuf};
use tracing::warn;
use url::{ParseError, Url};

/// A preprocessor for converting file name `README.md` to `index.md` since
/// `README.md` is the de facto index file in markdown-based documentation.
#[derive(Default)]
#[non_exhaustive]
pub struct IndexPreprocessor;

impl IndexPreprocessor {
    /// Name of this preprocessor.
    pub const NAME: &'static str = "index";

    /// Create a new `IndexPreprocessor`.
    pub fn new() -> Self {
        IndexPreprocessor
    }
}

impl Preprocessor for IndexPreprocessor {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let source_dir = ctx.root.join(&ctx.config.book.src);
        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut ch) = *section {
                // Rename README files to `index.md`
                if let Some(ref mut path) = ch.path {
                    if is_readme_file(&path) {
                        let mut index_md = source_dir.join(path.with_file_name("index.md"));
                        if index_md.exists() {
                            warn_readme_name_conflict(&path, &&mut index_md);
                        }

                        path.set_file_name("index.md");
                    }
                }
                // Fix inline links to README files
                let markdown_options = get_markdown_options(ctx);
                ch.content = fix_readme_links(&ch.content, &markdown_options).unwrap_or_default();
            }
        });

        Ok(book)
    }
}

fn warn_readme_name_conflict<P: AsRef<Path>>(readme_path: P, index_path: P) {
    let file_name = readme_path.as_ref().file_name().unwrap_or_default();
    let parent_dir = index_path
        .as_ref()
        .parent()
        .unwrap_or_else(|| index_path.as_ref());
    warn!(
        "It seems that there are both {:?} and index.md under \"{}\".",
        file_name,
        parent_dir.display()
    );
    warn!(
        "mdbook converts {:?} into index.html by default. It may cause",
        file_name
    );
    warn!("unexpected behavior if putting both files under the same directory.");
    warn!("To solve the warning, try to rearrange the book structure or disable");
    warn!("\"index\" preprocessor to stop the conversion.");
}

fn is_readme_file<P: AsRef<Path>>(path: P) -> bool {
    static_regex!(README, r"(?i)^readme$");

    README.is_match(
        path.as_ref()
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default(),
    )
}

fn get_markdown_options(ctx: &PreprocessorContext) -> MarkdownOptions {
    let html_config = ctx.config.html_config().unwrap_or_default();
    let mut markdown_options = MarkdownOptions::default();
    markdown_options.smart_punctuation = html_config.smart_punctuation;
    markdown_options.definition_lists = html_config.definition_lists;
    markdown_options.admonitions = html_config.admonitions;
    markdown_options
}

fn fix_readme_links(content: &str, markdown_options: &MarkdownOptions) -> Result<String> {
    let mut buf = String::with_capacity(content.len());

    let events = new_cmark_parser(content, markdown_options).map(|e| match e {
        Event::Start(Tag::Link {
            link_type:
                link_type @ (LinkType::Inline
                | LinkType::Reference
                | LinkType::Collapsed
                | LinkType::Shortcut),
            dest_url,
            title,
            id,
        }) if matches!(
            Url::parse(&dest_url),
            Err(ParseError::RelativeUrlWithoutBase)
        ) =>
        {
            Event::Start(Tag::Link {
                link_type: link_type,
                dest_url: {
                    let mut path = PathBuf::from(dest_url.as_ref());
                    if is_readme_file(&path) {
                        path.set_file_name("index.md");
                    }
                    path.to_string_lossy().into_owned().into()
                },
                title,
                id,
            })
        }
        e => e,
    });

    Ok(pulldown_cmark_to_cmark::cmark(events, &mut buf).map(|_| buf)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_stem_exactly_matches_readme_case_insensitively() {
        let path = "path/to/Readme.md";
        assert!(is_readme_file(path));

        let path = "path/to/README.md";
        assert!(is_readme_file(path));

        let path = "path/to/rEaDmE.md";
        assert!(is_readme_file(path));

        let path = "path/to/README.markdown";
        assert!(is_readme_file(path));

        let path = "path/to/README";
        assert!(is_readme_file(path));

        let path = "path/to/README-README.md";
        assert!(!is_readme_file(path));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn internal_readme_links_are_converted_to_index() {
        let opts = MarkdownOptions::default();

        let md = "[inline](path/to/README.md)";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "[inline](path/to\\index.md)"
        );

        let md = "[reference][link]\n\n[link]: path/to/README.md";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "[reference][link]\n\n[link]: path/to\\index.md"
        );

        let md = "[collapsed][]\n\n[collapsed]: path/to/README.md";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "[collapsed][]\n\n[collapsed]: path/to\\index.md"
        );

        let md = "[shortcut]\n\n[shortcut]: path/to/README.md";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "[shortcut]\n\n[shortcut]: path/to\\index.md"
        );
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn internal_readme_links_are_converted_to_index() {
        let opts = MarkdownOptions::default();

        let md = "[inline](path/to/README.md)";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "[inline](path/to/index.md)"
        );

        let md = "[reference][link]\n\n[link]: path/to/README.md";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "[reference][link]\n\n[link]: path/to/index.md"
        );

        let md = "[collapsed][]\n\n[collapsed]: path/to/README.md";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "[collapsed][]\n\n[collapsed]: path/to/index.md"
        );

        let md = "[shortcut]\n\n[shortcut]: path/to/README.md";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "[shortcut]\n\n[shortcut]: path/to/index.md"
        );
    }

    #[test]
    fn other_links_are_not_converted_to_index() {
        let opts = MarkdownOptions::default();

        let md = "[inline](https://example.com)";
        assert_eq!(fix_readme_links(md, &opts).unwrap(), md);

        let md = "[inline]()";
        assert_eq!(fix_readme_links(md, &opts).unwrap(), md);

        let md = "[reference][link]\n\n[link]: https://example.com";
        assert_eq!(fix_readme_links(md, &opts).unwrap(), md);

        let md = "[reference][unknown]";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "\\[reference\\]\\[unknown\\]"
        );

        let md = "[collapsed][]\n\n[collapsed]: https://example.com";
        assert_eq!(fix_readme_links(md, &opts).unwrap(), md);

        let md = "[collapsed][]";
        assert_eq!(
            fix_readme_links(md, &opts).unwrap(),
            "\\[collapsed\\]\\[\\]"
        );

        let md = "[shortcut]\n\n[shortcut]: https://example.com";
        assert_eq!(fix_readme_links(md, &opts).unwrap(), md);

        let md = "[shortcut]";
        assert_eq!(fix_readme_links(md, &opts).unwrap(), "\\[shortcut\\]");

        let md = "<https://example.com>";
        assert_eq!(fix_readme_links(md, &opts).unwrap(), md);

        let md = "<user@example.com>";
        assert_eq!(fix_readme_links(md, &opts).unwrap(), md);
    }
}
