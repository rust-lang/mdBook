use super::{Preprocessor, PreprocessorContext};
use book::Book;
use errors::*;
use serde_json;
use shlex::Shlex;
use std::io::{self, Read};
use std::process::{Child, Command, Stdio};

/// A custom preprocessor which will shell out to a 3rd-party program.
///
/// # Preprocessing
///
/// When the `supports_renderer()` method is executed, `CmdPreprocessor` will
/// execute the shell command `$cmd supports $renderer`. If the renderer is
/// supported, custom preprocessors should exit with a exit code of `0`,
/// any other exit code be considered as unsupported.
#[derive(Debug, Clone, PartialEq)]
pub struct CmdPreprocessor {
    name: String,
    cmd: String,
}

impl CmdPreprocessor {
    /// Create a new `CmdPreprocessor`.
    pub fn new(name: String, cmd: String) -> CmdPreprocessor {
        CmdPreprocessor { name, cmd }
    }

    /// A convenience function custom preprocessors can use to parse the input
    /// written to `stdin` by a `CmdRenderer`.
    pub fn parse_input<R: Read>(
        reader: R,
    ) -> Result<(PreprocessorContext, Book)> {
        serde_json::from_reader(reader)
            .chain_err(|| "Unable to parse the input")
    }

    fn write_input(
        &self,
        child: &mut Child,
        book: Book,
        ctx: &PreprocessorContext,
    ) {
        let mut stdin = child.stdin.take().expect("Child has stdin");
        let input = (ctx, book);

        if let Err(e) = serde_json::to_writer(&mut stdin, &input) {
            // Looks like the backend hung up before we could finish
            // sending it the render context. Log the error and keep going
            warn!("Error writing the RenderContext to the backend, {}", e);
        }

        // explicitly close the `stdin` file handle
        drop(stdin);
    }

    fn command(&self) -> Result<Command> {
        let mut words = Shlex::new(&self.cmd);
        let executable = match words.next() {
            Some(e) => e,
            None => bail!("Command string was empty"),
        };

        let mut cmd = Command::new(executable);

        for arg in words {
            cmd.arg(arg);
        }

        Ok(cmd)
    }
}

impl Preprocessor for CmdPreprocessor {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        let mut cmd = self.command()?;

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .chain_err(|| format!("Unable to start the \"{}\" preprocessor. Is it installed?", self.name()))?;

        self.write_input(&mut child, book, ctx);

        let output = child
            .wait_with_output()
            .chain_err(|| "Error waiting for the preprocessor to complete")?;

        trace!("{} exited with output: {:?}", self.cmd, output);
        ensure!(output.status.success(), "The preprocessor exited unsuccessfully");

        serde_json::from_slice(&output.stdout).chain_err(|| "Unable to parse the preprocessed book")
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        debug!("Checking if the \"{}\" preprocessor supports \"{}\"", self.name(), renderer);

        let mut cmd = match self.command() {
            Ok(c) => c,
            Err(e) => {
                warn!("Unable to create the command for the \"{}\" preprocessor, {}", self.name(), e);
                return false;
            }
        };

        let outcome = cmd
            .arg("supports")
            .arg(renderer)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map(|status| status.code() == Some(0));

        if let Err(ref e) = outcome {
            if e.kind() == io::ErrorKind::NotFound {
                warn!(
                    "The command wasn't found, is the \"{}\" preprocessor installed?",
                    self.name
                );
                warn!("\tCommand: {}", self.cmd);
            }
        }

        outcome.unwrap_or(false)
    }
}