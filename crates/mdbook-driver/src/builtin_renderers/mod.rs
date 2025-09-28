//! Built-in renderers.
//!
//! The HTML renderer can be found in the [`mdbook_html`] crate.

use anyhow::{Context, Result, bail};
use mdbook_core::utils::fs;
use mdbook_renderer::{RenderContext, Renderer};
use std::process::Stdio;
use tracing::{error, info, trace, warn};

pub use self::markdown_renderer::MarkdownRenderer;

mod markdown_renderer;

/// A generic renderer which will shell out to an arbitrary executable.
///
/// See <https://rust-lang.github.io/mdBook/for_developers/backends.html>
/// for a description of the renderer protocol.
#[derive(Debug, Clone, PartialEq)]
pub struct CmdRenderer {
    name: String,
    cmd: String,
}

impl CmdRenderer {
    /// Create a new `CmdRenderer` which will invoke the provided `cmd` string.
    pub fn new(name: String, cmd: String) -> CmdRenderer {
        CmdRenderer { name, cmd }
    }
}

impl Renderer for CmdRenderer {
    fn name(&self) -> &str {
        &self.name
    }

    fn render(&self, ctx: &RenderContext) -> Result<()> {
        info!("Invoking the \"{}\" renderer", self.name);

        let optional_key = format!("output.{}.optional", self.name);
        let optional = match ctx.config.get(&optional_key) {
            Ok(Some(value)) => value,
            Err(e) => bail!("expected bool for `{optional_key}`: {e}"),
            Ok(None) => false,
        };

        let _ = fs::create_dir_all(&ctx.destination);

        let mut cmd = crate::compose_command(&self.cmd, &ctx.root)?;
        let mut child = match cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(&ctx.destination)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                return crate::handle_command_error(
                    e, optional, "output", "backend", &self.name, &self.cmd,
                );
            }
        };

        let mut stdin = child.stdin.take().expect("Child has stdin");
        if let Err(e) = serde_json::to_writer(&mut stdin, &ctx) {
            // Looks like the backend hung up before we could finish
            // sending it the render context. Log the error and keep going
            warn!("Error writing the RenderContext to the backend, {}", e);
        }

        // explicitly close the `stdin` file handle
        drop(stdin);

        let status = child
            .wait()
            .with_context(|| "Error waiting for the backend to complete")?;

        trace!("{} exited with output: {:?}", self.cmd, status);

        if !status.success() {
            error!("Renderer exited with non-zero return code.");
            bail!("The \"{}\" renderer failed", self.name);
        } else {
            Ok(())
        }
    }
}
