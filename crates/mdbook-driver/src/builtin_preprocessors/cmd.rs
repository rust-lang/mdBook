use anyhow::{Context, Result, ensure};
use mdbook_core::book::Book;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Stdio};
use tracing::{debug, trace, warn};

/// A custom preprocessor which will shell out to a 3rd-party program.
///
/// # Preprocessing Protocol
///
/// When the `supports_renderer()` method is executed, `CmdPreprocessor` will
/// execute the shell command `$cmd supports $renderer`. If the renderer is
/// supported, custom preprocessors should exit with a exit code of `0`,
/// any other exit code be considered as unsupported.
///
/// The `run()` method is implemented by passing a `(PreprocessorContext, Book)`
/// tuple to the spawned command (`$cmd`) as JSON via `stdin`. Preprocessors
/// should then "return" a processed book by printing it to `stdout` as JSON.
/// For convenience, the `CmdPreprocessor::parse_input()` function can be used
/// to parse the input provided by `mdbook`.
///
/// Exiting with a non-zero exit code while preprocessing is considered an
/// error. `stderr` is passed directly through to the user, so it can be used
/// for logging or emitting warnings if desired.
///
/// # Examples
///
/// An example preprocessor is available in this project's `examples/`
/// directory.
#[derive(Debug, Clone, PartialEq)]
pub struct CmdPreprocessor {
    name: String,
    cmd: String,
    root: PathBuf,
    optional: bool,
}

impl CmdPreprocessor {
    /// Create a new `CmdPreprocessor`.
    pub fn new(name: String, cmd: String, root: PathBuf, optional: bool) -> CmdPreprocessor {
        CmdPreprocessor {
            name,
            cmd,
            root,
            optional,
        }
    }

    fn write_input_to_child(&self, child: &mut Child, book: &Book, ctx: &PreprocessorContext) {
        let stdin = child.stdin.take().expect("Child has stdin");

        if let Err(e) = self.write_input(stdin, book, ctx) {
            // Looks like the backend hung up before we could finish
            // sending it the render context. Log the error and keep going
            warn!("Error writing the RenderContext to the backend, {}", e);
        }
    }

    fn write_input<W: Write>(
        &self,
        writer: W,
        book: &Book,
        ctx: &PreprocessorContext,
    ) -> Result<()> {
        serde_json::to_writer(writer, &(ctx, book)).map_err(Into::into)
    }

    /// The command this `Preprocessor` will invoke.
    pub fn cmd(&self) -> &str {
        &self.cmd
    }
}

impl Preprocessor for CmdPreprocessor {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        let mut cmd = crate::compose_command(&self.cmd, &ctx.root)?;

        let mut child = match cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .current_dir(&self.root)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                crate::handle_command_error(
                    e,
                    self.optional,
                    "preprocessor",
                    "preprocessor",
                    &self.name,
                    &self.cmd,
                )?;
                // This should normally not be reached, since the validation
                // for NotFound should have already happened when running the
                // "supports" command.
                return Ok(book);
            }
        };

        self.write_input_to_child(&mut child, &book, ctx);

        let output = child.wait_with_output().with_context(|| {
            format!(
                "Error waiting for the \"{}\" preprocessor to complete",
                self.name
            )
        })?;

        trace!("{} exited with output: {:?}", self.cmd, output);
        ensure!(
            output.status.success(),
            format!(
                "The \"{}\" preprocessor exited unsuccessfully with {} status",
                self.name, output.status
            )
        );

        serde_json::from_slice(&output.stdout).with_context(|| {
            format!(
                "Unable to parse the preprocessed book from \"{}\" processor",
                self.name
            )
        })
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        debug!(
            "Checking if the \"{}\" preprocessor supports \"{}\"",
            self.name(),
            renderer
        );

        let mut cmd = crate::compose_command(&self.cmd, &self.root)?;

        match cmd
            .arg("supports")
            .arg(renderer)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(&self.root)
            .status()
        {
            Ok(status) => Ok(status.code() == Some(0)),
            Err(e) => {
                crate::handle_command_error(
                    e,
                    self.optional,
                    "preprocessor",
                    "preprocessor",
                    &self.name,
                    &self.cmd,
                )?;
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MDBook;
    use std::path::Path;

    fn guide() -> MDBook {
        let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../guide");
        MDBook::load(example).unwrap()
    }

    #[test]
    fn round_trip_write_and_parse_input() {
        let md = guide();
        let cmd = CmdPreprocessor::new(
            "test".to_string(),
            "test".to_string(),
            md.root.clone(),
            false,
        );
        let ctx = PreprocessorContext::new(
            md.root.clone(),
            md.config.clone(),
            "some-renderer".to_string(),
        );

        let mut buffer = Vec::new();
        cmd.write_input(&mut buffer, &md.book, &ctx).unwrap();

        let (got_ctx, got_book) = mdbook_preprocessor::parse_input(buffer.as_slice()).unwrap();

        assert_eq!(got_book, md.book);
        assert_eq!(got_ctx, ctx);
    }
}
