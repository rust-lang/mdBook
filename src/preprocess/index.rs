use regex::Regex;
use std::path::Path;

use crate::errors::*;

use super::{Preprocessor, PreprocessorContext};
use crate::book::{Book, BookItem};

/// A preprocessor for converting file name `README.md` to `index.md` since
/// `README.md` is the de facto index file in markdown-based documentation.
#[derive(Default)]
pub struct IndexPreprocessor;

impl IndexPreprocessor {
    pub(crate) const NAME: &'static str = "index";

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
                if let Some(ref mut path) = ch.path {
                    if is_readme_file(&path) {
                        let mut index_md = source_dir.join(path.with_file_name("index.md"));
                        if index_md.exists() {
                            warn_readme_name_conflict(&path, &&mut index_md);
                        }

                        path.set_file_name("index.md");
                    }
                }
                ch.content = replace_readme_in_string(&ch.content);
            }
        });

        Ok(book)
    }
}

fn replace_readme_in_string(content: &str) -> String {
    lazy_static! {
        static ref RE_INLINE: Regex = Regex::new(
            r"(?ix)        #ignorecase, allow regex definition eXtended on multiple lines
            \[([^\[]+)\]       #[name_of_link]
            \s?                #optional whitespaces
            \(                 #open parenthesis
            ([^\[\s]*)         #start of path : url/blabla/
            (readme.md)        #part that will be replaced by index.md
            (?:                #BEGIN optional part, '?:' ignores capture of this whole (group)
                \s+                #whitespace between path and optional title
                ([^\[]*)           #optional title
            )                  #END optional part
            \s*                #trailing whitespaces
            \)                 #close parenthesis"
        )
        .unwrap();
        static ref RE_REFERENCE: Regex = Regex::new(
            r"(?ix)        #ignorecase, allow regex definition eXtended on multiple lines
        ^                  #start of line
        (\s*)              #optional padding whitespaces
        \[([^\[]+)\]:      #[name_of_link]:
        \s*                #optional whitespaces
        ([^\[\s]*)         #start of path
        (readme.md)        #part that will be replaced by index.md
        (?:                #BEGIN optional part, '?:' ignores capture of this whole (group)
            \s+                #whitespace between path and optional title
            ([^\[]*)           #optional title
        )                  #END optional part
        \s*                #trailing whitespaces
        $                  #end of line"
        )
        .unwrap();
    }
    let content = RE_INLINE.replace_all(&content, "[$1](${2}index.md $4)");
    let content = RE_REFERENCE.replace_all(&content, "${1}[$2]: ${3}index.md $5");
    content.to_string()
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
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)^readme$").unwrap();
    }
    RE.is_match(
        path.as_ref()
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default(),
    )
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
    fn do_not_replace_readme_in_normal_string_test() {
        let expected = "content: ( index.md)";

        let content = "content: ( Readme.md )";
        assert_ne!(replace_readme_in_string(content), expected);

        let content = "content: ( README.md )";
        assert_ne!(replace_readme_in_string(content), expected);

        let content = "content: ( rEaDmE.md )";
        assert_ne!(replace_readme_in_string(content), expected);

        let content = "content: ( README-README.md )";
        assert_ne!(replace_readme_in_string(content), expected);
    }
    #[test]
    fn replace_readme_in_inline_link_test() {
        let expected = "[content](./bla/index.md )";
        let expected_with_title = "[content](./bla/index.md \"title\" )";

        let content = "[content](./bla/Readme.md )";
        assert_eq!(replace_readme_in_string(content), expected);

        let content = "[content](./bla/README.md )";
        assert_eq!(replace_readme_in_string(content), expected);

        let content = "[content](./bla/rEaDmE.md )";
        assert_eq!(replace_readme_in_string(content), expected);

        let content = "[content](./bla/rEaDmE.md \"title\" )";
        assert_eq!(replace_readme_in_string(content), expected_with_title);

        let content = "[content](./bla/README-README.md )";
        assert_ne!(replace_readme_in_string(content), expected);
    }
    #[test]
    fn replace_readme_in_footnote_link_test() {
        let expected = "[content]: ./bla/index.md ";
        let expected_with_title = "[content]: ./bla/index.md \"title\" ";

        let content = "[content]: ./bla/Readme.md ";
        assert_eq!(replace_readme_in_string(content), expected);

        let content = "[content]: ./bla/README.md ";
        assert_eq!(replace_readme_in_string(content), expected);

        let content = "[content]: ./bla/rEaDmE.md ";
        assert_eq!(replace_readme_in_string(content), expected);

        let content = "[content]: ./bla/rEaDmE.md \"title\" ";
        assert_eq!(replace_readme_in_string(content), expected_with_title);

        let content = "[content]: ./bla/README-README.md ";
        assert_ne!(replace_readme_in_string(content), expected);
    }
}
