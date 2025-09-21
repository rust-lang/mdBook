use anyhow::{Context, Result};
use mdbook_core::utils::fs;
use mdbook_renderer::{RenderContext, Renderer};
use tracing::trace;

/// A renderer to output the Markdown after the preprocessors have run. Mostly useful
/// when debugging preprocessors.
#[derive(Default)]
#[non_exhaustive]
pub struct MarkdownRenderer;

impl MarkdownRenderer {
    /// Create a new `MarkdownRenderer` instance.
    pub fn new() -> Self {
        MarkdownRenderer
    }
}

impl Renderer for MarkdownRenderer {
    fn name(&self) -> &str {
        "markdown"
    }

    fn render(&self, ctx: &RenderContext) -> Result<()> {
        let destination = &ctx.destination;
        let book = &ctx.book;

        if destination.exists() {
            fs::remove_dir_content(destination)
                .with_context(|| "Unable to remove stale Markdown output")?;
        }

        trace!("markdown render");
        for ch in book.chapters() {
            let path = ctx
                .destination
                .join(ch.path.as_ref().expect("Checked path exists before"));
            fs::write(path, &ch.content)?;
        }

        fs::create_dir_all(destination)
            .with_context(|| "Unexpected error when constructing destination path")?;

        Ok(())
    }
}
