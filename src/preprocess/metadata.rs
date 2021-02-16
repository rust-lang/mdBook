use crate::errors::*;
use regex::Regex;
use std::ops::Range;

use super::{Preprocessor, PreprocessorContext};
use crate::book::{Book, BookItem};

/// A preprocessor for reading TOML front matter from a markdown file. Special
/// fields are included in the `index.hbs` file for handlebars.js templating and
/// are:
/// - `author` - For setting the author meta tag.
/// - `title` - For overwritting the title tag.
/// - `description` - For setting the description meta tag.
/// - `keywords` - For setting the keywords meta tag.
#[derive(Default)]
pub struct MetadataPreprocessor;

impl MetadataPreprocessor {
    pub(crate) const NAME: &'static str = "metadata";

    /// Create a new `MetadataPreprocessor`.
    pub fn new() -> Self {
        MetadataPreprocessor
    }
}

impl Preprocessor for MetadataPreprocessor {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut ch) = *section {
                if let Some(m) = Match::find_metadata(&ch.content) {
                    if let Ok(mut meta) = toml::from_str(&ch.content[m.range]) {
                        ch.chapter_config.append(&mut meta);
                        ch.content = String::from(&ch.content[m.end..]);
                    };
                }
            }
        });
        Ok(book)
    }
}

struct Match {
    range: Range<usize>,
    end: usize,
}

impl Match {
    fn find_metadata(contents: &str) -> Option<Match> {
        // lazily compute following regex
        // r"\A-{3,}\n(?P<metadata>.*?)^{3,}\n"
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?xms)          # insignificant whitespace mode and multiline
                \A-{3,}\n         # match a horizontal rule at the start of the content
                (?P<metadata>.*?) # name the match between horizontal rules metadata
                ^-{3,}\n          # match a horizontal rule
                "
            )
            .unwrap();
        };
        if let Some(mat) = RE.captures(contents) {
            // safe to unwrap as we know there is a match
            let metadata = mat.name("metadata").unwrap();
            Some( Match {
                range: metadata.start()..metadata.end(),
                end: mat.get(0).unwrap().end(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_metadata_not_at_start() {
        let s = "\
        content\n\
        ---
        author = \"Adam\"
        title = \"Blog Post #1\"
        keywords = [
            \"rust\",
            \"blog\",
        ]
        date = \"2021/02/15\"
        modified = \"2021/02/16\"\n\
        ---
        content
        ";
        if let Some(_) = Match::find_metadata(s) {
            panic!()
        }
    }

    #[test]
    fn test_find_metadata_at_start() {
        let s = "\
        ---
        author = \"Adam\"
        title = \"Blog Post #1\"
        keywords = [
            \"rust\",
            \"blog\",
        ]
        date = \"2021/02/15\"
        description = \"My rust blog.\"
        modified = \"2021/02/16\"\n\
        ---\n\
        content
        ";
        if let None = Match::find_metadata(s) {
            panic!()
        }
    }

    #[test]
    fn test_find_metadata_partial_metadata() {
        let s = "\
        ---
        author = \"Adam\n\
        content
        ";
        if let Some(_) = Match::find_metadata(s) {
            panic!()
        }
    }

    #[test]
    fn test_find_metadata_not_metadata() {
        type Map = serde_json::Map<String, serde_json::Value>;
        let s = "\
        ---
        This is just standard content that happens to start with a line break
        and has a second line break in the text.\n\
        ---
        followed by more content
        ";
        if let Some(m) = Match::find_metadata(s) {
            if let Ok(_) = toml::from_str::<Map>(&s[m.range]) {
                panic!()
            }
        }
    }

    #[test]
    fn test_parse_metadata() {
        let metadata: serde_json::Map<String, serde_json::Value> = toml::from_str(
            "author = \"Adam\"
        title = \"Blog Post #1\"
        keywords = [
            \"Rust\",
            \"Blog\",
        ]
        date = \"2021/02/15\"
        ").unwrap();
        let mut map = serde_json::Map::<String, serde_json::Value>::new();
        map.insert("author".to_string(), json!("Adam"));
        map.insert("title".to_string(), json!("Blog Post #1"));
        map.insert("keywords".to_string(), json!(vec!["Rust", "Blog"]));
        map.insert("date".to_string(), json!("2021/02/15"));
        assert_eq!(metadata, map)
    }
}
