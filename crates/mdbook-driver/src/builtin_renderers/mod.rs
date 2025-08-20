//! Built-in renderers.
//!
//! The HTML renderer can be found in the [`mdbook_html`] crate.

use anyhow::{Context, Result, bail};
use log::{error, info, trace, warn};
use mdbook_renderer::{RenderContext, Renderer};
use std::fs;
use std::process::Stdio;

pub use self::markdown_renderer::MarkdownRenderer;

mod markdown_renderer;

/// A generic renderer which will shell out to an arbitrary executable.
///
/// # Rendering Protocol
///
/// When the renderer's `render()` method is invoked, `CmdRenderer` will spawn
/// the `cmd` as a subprocess. The `RenderContext` is passed to the subprocess
/// as a JSON string (using `serde_json`).
///
/// > **Note:** The command used doesn't necessarily need to be a single
/// > executable (i.e. `/path/to/renderer`). The `cmd` string lets you pass
/// > in command line arguments, so there's no reason why it couldn't be
/// > `python /path/to/renderer --from mdbook --to epub`.
///
/// Anything the subprocess writes to `stdin` or `stdout` will be passed through
/// to the user. While this gives the renderer maximum flexibility to output
/// whatever it wants, to avoid spamming users it is recommended to avoid
/// unnecessary output.
///
/// To help choose the appropriate output level, the `RUST_LOG` environment
/// variable will be passed through to the subprocess, if set.
///
/// If the subprocess wishes to indicate that rendering failed, it should exit
/// with a non-zero return code.
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
