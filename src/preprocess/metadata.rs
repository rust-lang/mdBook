use crate::errors::*;
use regex::{CaptureMatches, Captures, Regex};

use super::{Preprocessor, PreprocessorContext};
use crate::book::{Book, BookItem};

/// A preprocessor for reading TOML front matter from a markdown file. The supported
/// fields are:
/// - `author` - For setting the author meta tag.
/// - `title` - For overwritting the title tag.
/// - `description` - For setting the description meta tag.
/// - `keywords` - For setting the keywords meta tag.
/// - `date` - The date the file was created, creates a handlebar.js vairable {{date}}.
/// - `modified` - The date the file was modified, creates a handlebar.js vairable {{modified}}.
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
                let (metadata, content) = collect(&ch.content);
                ch.content = content;
                ch.chapter_config.append(&mut metadata.to_map());
            }
        });
        Ok(book)
    }
}

fn collect(s: &str) -> (Metadata, String) {
    let mut end_index = 0;
    let mut replaced = String::new();

    let metadata: Metadata = if let Some(metadata) = find_metadata(s).next() {
        match toml::from_str(metadata.text) {
            Ok(meta) => {
                end_index += metadata.end_index;
                meta
            }
            _ => Metadata::default(),
        }
    } else {
        Metadata::default()
    };

    replaced.push_str(&s[end_index..]);
    (metadata, replaced)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct Metadata {
    author: Option<String>,
    title: Option<String>,
    date: Option<String>,
    keywords: Option<Vec<String>>,
    description: Option<String>,
    modified: Option<String>,
}

impl Metadata {
    fn to_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        if let Some(author) = self.author {
            map.insert("author".to_string(), json!(author));
        }
        if let Some(title) = self.title {
            map.insert("title".to_string(), json!(title));
        }
        if let Some(date) = self.date {
            map.insert("date".to_string(), json!(date));
        }
        if let Some(keywords) = self.keywords {
            map.insert("keywords".to_string(), json!(keywords));
        }
        if let Some(modified) = self.modified {
            map.insert("modified".to_string(), json!(modified));
        }
        if let Some(description) = self.description {
            map.insert("description".to_string(), json!(description));
        }
        map
    }
}

impl Default for Metadata {
    fn default() -> Metadata {
        Metadata {
            author: None,
            title: None,
            date: None,
            keywords: None,
            modified: None,
            description: None,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
struct MetadataItem<'a> {
    end_index: usize,
    text: &'a str,
}

impl<'a> MetadataItem<'a> {
    fn from_capture(cap: Captures<'a>) -> Option<MetadataItem<'a>> {
        if let Some(mat) = cap.name("metadata") {
            let full_match = cap.get(0).unwrap();
            if full_match.start() == 0 {
                return Some(MetadataItem {
                    end_index: full_match.end(),
                    text: mat.as_str(),
                });
            }
        }
        None
    }
}

struct MetadataIter<'a>(CaptureMatches<'a, 'a>);

impl<'a> Iterator for MetadataIter<'a> {
    type Item = MetadataItem<'a>;
    fn next(&mut self) -> Option<MetadataItem<'a>> {
        for cap in &mut self.0 {
            if let Some(inc) = MetadataItem::from_capture(cap) {
                return Some(inc);
            }
        }
        None
    }
}

fn find_metadata(contents: &str) -> MetadataIter<'_> {
    // lazily compute following regex
    // r"^-{3,}\n(?P<metadata>.*?)^{3,}\n"
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?xms)          # insignificant whitespace mode and multiline
            ^-{3,}\n          # match a horizontal rule
            (?P<metadata>.*?) # name the match between horizontal rules metadata
            ^-{3,}\n          # match a horizontal rule
            "
        )
        .unwrap();
    }
    MetadataIter(RE.captures_iter(contents))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_not_at_start() {
        let start = "\
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
        assert_eq!(collect(start).1, start);
    }

    #[test]
    fn test_collect_at_start() {
        let start = "\
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
        let end = "\
        content
        ";
        assert_eq!(collect(start).1, end);
    }

    #[test]
    fn test_collect_partial_metadata() {
        let start = "\
        ---
        author = \"Adam\"\n\
        ---\n\
        content
        ";
        let end = "\
        content
        ";
        assert_eq!(collect(start).1, end);
        assert_eq!(
            collect(start).0,
            Metadata {
                author: Some("Adam".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_collect_unsupported_metadata() {
        let start = "\
        ---
        author: \"Adam\"
        unsupported_field: \"text\"\n\
        ---
        followed by more content
        ";
        assert_eq!(collect(start).1, start);
    }

    #[test]
    fn test_collect_not_metadata() {
        let start = "\
        ---
        This is just standard content that happens to start with a line break
        and has a second line break in the text.\n\
        ---
        followed by more content
        ";
        assert_eq!(collect(start).1, start);
    }

    #[test]
    fn test_metadata_to_map() {
        let metadata: Metadata = toml::from_str(
            "author = \"Adam\"
        title = \"Blog Post #1\"
        keywords = [
            \"Rust\",
            \"Blog\",
        ]
        date = \"2021/02/15\"
        description = \"My rust blog.\"
        modified = \"2021/02/16\" ",
        )
        .unwrap();
        let mut map = serde_json::Map::new();
        map.insert("author".to_string(), json!("Adam"));
        map.insert("title".to_string(), json!("Blog Post #1"));
        map.insert("keywords".to_string(), json!(vec!["Rust", "Blog"]));
        map.insert("date".to_string(), json!("2021/02/15"));
        map.insert("description".to_string(), json!("My rust blog."));
        map.insert("modified".to_string(), json!("2021/02/16"));
        assert_eq!(metadata.to_map(), map)
    }
}
