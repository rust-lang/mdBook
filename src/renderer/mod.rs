pub use self::html_handlebars::HtmlHandlebars;

mod html_handlebars;

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use serde_json;

use errors::*;
use config::Config;
use book::{Book, MDBook};

pub trait Renderer {
    fn name(&self) -> &str;
    fn render(&self, book: &MDBook) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderContext {
    pub version: String,
    pub root: PathBuf,
    pub book: Book,
    pub config: Config,
    pub destination: PathBuf,
}

impl RenderContext {
    pub fn new<P, Q>(root: P, book: Book, config: Config, destination: Q) -> RenderContext
    where
        P: Into<PathBuf>,
        Q: Into<PathBuf>,
    {
        RenderContext {
            book: book,
            config: config,
            version: env!("CARGO_PKG_VERSION").to_string(),
            root: root.into(),
            destination: destination.into(),
        }
    }

    pub fn source_dir(&self) -> PathBuf {
        self.root.join(&self.config.book.src)
    }

    pub fn build_dir(&self) -> PathBuf {
        self.root.join(&self.config.build.build_dir)
    }

    pub fn from_json<R: Read>(reader: R) -> Result<RenderContext> {
        serde_json::from_reader(reader)
            .chain_err(|| "Unable to deserialize the `RenderContext`")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CmdRenderer {
    name: String,
    cmd: String,
    destination: PathBuf,
}

impl CmdRenderer {
    pub fn new<P: Into<PathBuf>>(name: String, cmd: String, destination: P) -> CmdRenderer {
        let destination = destination.into();
        CmdRenderer {
            name,
            cmd,
            destination,
        }
    }
}

impl Renderer for CmdRenderer {
    fn name(&self) -> &str {
        &self.name
    }

    fn render(&self, book: &MDBook) -> Result<()> {
        info!("Invoking the \"{}\" renderer", self.cmd);
        let ctx = RenderContext::new(
            &book.root,
            book.book.clone(),
            book.config.clone(),
            &self.destination,
        );

        let mut child = Command::new(&self.cmd)
            .stdin(Stdio::piped())
            .spawn()
            .chain_err(|| "Unable to start the renderer")?;

        serde_json::to_writer(
            child.stdin.as_mut().expect("stdin is always attached"),
            &ctx,
        ).chain_err(|| "Error occurred while sending the render context to the renderer")?;

        let status = child.wait()?;
        trace!("{} exited with output: {:?}", self.cmd, status);

        if !status.success() {
            error!("Renderer exited with non-zero return code.");
            bail!("The \"{}\" renderer failed", self.cmd);
        } else {
            Ok(())
        }
    }
}
