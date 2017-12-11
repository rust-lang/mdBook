pub use self::html_handlebars::HtmlHandlebars;

mod html_handlebars;

use std::path::PathBuf;
use std::process::{Command, Stdio};
use serde_json;

use errors::*;
use config::Config;
use book::{Book, MDBook};


pub trait Renderer {
    fn render(&self, book: &MDBook) -> Result<()>;
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderContext {
    pub version: &'static str,
    pub root: PathBuf,
    pub book: Book,
    pub config: Config,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlternateRenderer {
    cmd: String,
}

impl AlternateRenderer {
    pub fn new<S: Into<String>>(cmd: S) -> AlternateRenderer {
        AlternateRenderer { cmd: cmd.into() }
    }
}

impl Renderer for AlternateRenderer {
    fn render(&self, book: &MDBook) -> Result<()> {
        info!("Invoking the \"{}\" renderer", self.cmd);
        let ctx = RenderContext::new(&book.root, book.book.clone(), book.config.clone());

        let mut child = Command::new(&self.cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .chain_err(|| "Unable to start the renderer")?;

        serde_json::to_writer(
            child.stdin.as_mut().expect("stdin is always attached"),
            &ctx,
        ).chain_err(|| "Error occurred while sending the render context to the renderer")?;

        let output = child.wait_with_output()?;
        trace!("{} exited with output: {:?}", self.cmd, output);

        if !output.status.success() {
            error!("Renderer exited with non-zero return code.");
            bail!("Alternate renderer failed");
        } else {
            Ok(())
        }
    }
}

impl RenderContext {
    pub fn new<P: Into<PathBuf>>(root: P, book: Book, config: Config) -> RenderContext {
        RenderContext {
            book: book,
            config: config,
            version: env!("CARGO_PKG_VERSION"),
            root: root.into(),
        }
    }
}
