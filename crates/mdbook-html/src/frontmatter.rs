//! Frontmatter parsing support for mdBook.
//!
//! Extracts YAML frontmatter from markdown content and injects
//! Open Graph / Twitter Card metadata into the Handlebars template context.

use serde::Deserialize;
use serde_json::json;

/// Parsed YAML frontmatter fields.
#[derive(Deserialize, Debug)]
pub(crate) struct FrontMatter {
    /// Page title for OG/Twitter meta tags.
    pub title: String,
    /// Page description for OG/Twitter meta tags.
    pub description: String,
    /// Featured image URL for OG/Twitter meta tags.
    pub featured_image_url: String,
}

/// Strips YAML frontmatter (between `---` markers) from content,
/// returning the content without the frontmatter block.
pub(crate) fn strip_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }
    // Find the closing `---` after the opening one
    let after_open = &trimmed[3..];
    if let Some(end) = after_open.find("\n---") {
        // Skip past the closing `---` and any trailing newline
        let rest = &after_open[end + 4..];
        rest.trim_start_matches('\n').to_string()
    } else {
        content.to_string()
    }
}

/// Parses YAML frontmatter from content and injects OG metadata
/// into the Handlebars template context data map.
pub(crate) fn inject_frontmatter_data(
    content: &str,
    data: &mut serde_json::Map<String, serde_json::Value>,
) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return;
    }
    let after_open = &trimmed[3..];
    let Some(end) = after_open.find("\n---") else {
        return;
    };
    let yaml_str = &after_open[..end];

    match serde_yml::from_str::<FrontMatter>(yaml_str) {
        Ok(fm) => {
            data.insert("is_frontmatter".to_owned(), json!(true));
            data.insert("og_title".to_owned(), json!(fm.title));
            data.insert("og_description".to_owned(), json!(fm.description));
            data.insert("og_image_url".to_owned(), json!(fm.featured_image_url));
        }
        Err(e) => {
            eprintln!("Frontmatter: deserialization error: {e:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter() {
        let input = "---\ntitle: \"Hello\"\n---\n# Content";
        assert_eq!(strip_frontmatter(input), "# Content");
    }

    #[test]
    fn test_strip_no_frontmatter() {
        let input = "# Just content";
        assert_eq!(strip_frontmatter(input), "# Just content");
    }

    #[test]
    fn test_inject_frontmatter_data() {
        let input = "---\ntitle: \"My Title\"\ndescription: \"My Desc\"\nfeatured_image_url: \"https://example.com/img.png\"\n---\n# Content";
        let mut data = serde_json::Map::new();
        inject_frontmatter_data(input, &mut data);
        assert_eq!(data["is_frontmatter"], json!(true));
        assert_eq!(data["og_title"], json!("My Title"));
        assert_eq!(data["og_description"], json!("My Desc"));
        assert_eq!(data["og_image_url"], json!("https://example.com/img.png"));
    }

    #[test]
    fn test_inject_no_frontmatter() {
        let input = "# Just content";
        let mut data = serde_json::Map::new();
        inject_frontmatter_data(input, &mut data);
        assert!(!data.contains_key("is_frontmatter"));
    }
}
