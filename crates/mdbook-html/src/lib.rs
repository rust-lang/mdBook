//! mdBook HTML renderer.

mod html_handlebars;
pub mod theme;

pub use html_handlebars::HtmlHandlebars;
use mdbook_core::config::HtmlConfig;
use mdbook_markdown::HtmlRenderOptions;
use std::path::Path;

/// Creates an [`HtmlRenderOptions`] from the given config.
pub fn html_render_options_from_config<'a>(
    path: &'a Path,
    config: &HtmlConfig,
) -> HtmlRenderOptions<'a> {
    let mut options = HtmlRenderOptions::new(path);
    options.markdown_options.smart_punctuation = config.smart_punctuation;
    options
}
