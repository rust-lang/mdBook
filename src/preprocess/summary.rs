use std::path::Path;

/// A preprocessor dedicated just to the summary to resolve links
#[derive(Default)]
pub struct SummaryPreprocessor;

impl SummaryPreprocessor {
    /// Preprocess Summary to resolve links.
    pub fn resolve(src_dir: &Path, content: &str) -> String {
        let mut title = String::from("SUMMARY.md");
        super::links::replace_all(content, src_dir, src_dir, 0, &mut title)
    }
}
