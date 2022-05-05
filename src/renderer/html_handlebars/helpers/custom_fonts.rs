use crate::errors::Result;
use lewp_css::{
    domain::{
        at_rules::font_face::{FontFaceAtRule, FontUrlSource, Source},
        CssRule,
    },
    Stylesheet,
};
use std::path::{Path, PathBuf};

/// Parses the given file_name and extracts the given font file names.
pub fn find_custom_font_files(css_file_name: &Path) -> Result<Vec<PathBuf>> {
    let stylesheet = std::fs::read_to_string(css_file_name)?;
    let stylesheet = Stylesheet::parse(&stylesheet).unwrap_or_else(|_| {
        panic!(
            "Stylesheet: \"{}\" could not be parsed!",
            css_file_name.display()
        )
    });
    let css_rules = stylesheet.rules.0;
    let mut file_names = vec![];
    for rule in css_rules {
        match rule {
            CssRule::FontFace(font) => {
                if let Some(f) = extract_font_file_name(font) {
                    file_names.push(f);
                }
            }
            _ => continue,
        }
    }
    Ok(file_names)
}

/// Extracts the first file name given in an URL statement from the rule.
pub fn extract_font_file_name(at_rule: FontFaceAtRule) -> Option<PathBuf> {
    at_rule.sources.as_ref()?;
    match at_rule.sources.unwrap().get(0) {
        Some(Source::Url(FontUrlSource { url, .. })) => Some(PathBuf::from(url.0.clone())),
        _ => None,
    }
}
