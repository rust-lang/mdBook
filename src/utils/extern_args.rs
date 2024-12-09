//! Get "compiler" args from cargo

use crate::errors::*;
use log::info;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the arguments needed to invoke rustc so it can find external crates
/// when invoked by rustdoc to compile doctests.
///
/// It seems the `-L <libraryPath>` and `--extern <lib>=<pathInDeps>` args are sufficient.
///
/// Cargo doesn't expose a stable API to get this information.  
/// `cargo metadata` does not include the hash suffix in `<pathInDeps>`.
/// But it does leak when doing a build in verbose mode.
/// So we force a cargo build, capture the console output and parse the args therefrom.
///
/// Example:
/// ```rust
///
/// use mdbook::utils::extern_args::ExternArgs;
/// # use mdbook::errors::*;
///
/// # fn main() -> Result<()> {
/// // Get cargo to say what the compiler args need to be...
/// let proj_root = std::env::current_dir()?;    // or other path to `Cargo.toml`
/// let mut extern_args = ExternArgs::new();
/// extern_args.load(&proj_root)?;
///
/// // then, when actually invoking rustdoc or some other compiler-like tool...
/// 
/// assert!(extern_args.get_args().iter().any(|e| e == "-L")); // args contains "-L".to_string()
/// assert!(extern_args.get_args().iter().any(|e| e == "--extern"));
/// # Ok(())
/// # }
/// ```

#[derive(Debug)]
pub struct ExternArgs {
    suffix_args: Vec<String>,
}

impl ExternArgs {
    /// simple constructor
    pub fn new() -> Self {
        ExternArgs {
            suffix_args: vec![],
        }
    }

    /// Run a `cargo build` to see what args Cargo is using for library paths and extern crates.
    /// Touch a source file to ensure something is compiled and the args will be visible.
    ///
    /// >>>Future research: see whether `cargo check` can be used instead.  It emits the `--extern`s
    /// with `.rmeta` instead of `.rlib`, and the compiler can't actually use those
    /// when compiling a doctest.  But  perhaps simply changing the file extension would work?
    pub fn load(&mut self, proj_root: &Path) -> Result<&Self> {
        // touch (change) a file in the project to force check to do something

        for fname in ["lib.rs", "main.rs"] {
            let try_path: PathBuf = [&proj_root.to_string_lossy(), "src", fname]
                .iter()
                .collect();
            let f = File::options().append(true).open(&try_path)?;
            f.set_modified(std::time::SystemTime::now())?;
            break;
            // file should be closed when f goes out of scope at bottom of this loop
        }

        let mut cmd = Command::new("cargo");
        cmd.current_dir(&proj_root).arg("build").arg("--verbose");

        info!("running {:?}", cmd);
        let output = cmd.output()?;

        if !output.status.success() {
            bail!("Exit status {} from {:?}", output.status, cmd);
        }

        let cmd_resp: &str = std::str::from_utf8(&output.stderr)?;
        self.parse_response(&cmd_resp)?;

        Ok(self)
    }

    /// Parse response stdout+stderr response from `cargo build`
    /// into arguments we can use to invoke rustdoc.
    ///
    /// >>> This parser is broken, doesn't handle arg values with embedded spaces (single quoted).
    /// Fortunately, the args we care about (so far) don't have those kinds of values.
    pub fn parse_response(&mut self, buf: &str) -> Result<()> {
        for l in buf.lines() {
            if let Some(_i) = l.find(" Running ") {
                let args_seg: &str = l.split('`').skip(1).take(1).collect::<Vec<_>>()[0]; // sadly, cargo decorates string with backticks
                let mut arg_iter = args_seg.split_whitespace();

                while let Some(arg) = arg_iter.next() {
                    match arg {
                        "-L" | "--library-path" | "--extern" => {
                            self.suffix_args.push(arg.to_owned());
                            self.suffix_args
                                .push(arg_iter.next().unwrap_or("").to_owned());
                        }
                        _ => {}
                    }
                }
            };
        }

        Ok(())
    }

    /// get a list of (-L and --extern) args used to invoke rustdoc.
    pub fn get_args(&self) -> Vec<String> {
        self.suffix_args.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_response_parses_string() -> Result<()> {
        let resp = std::fs::read_to_string("tests/t1.txt")?;
        let mut ea = ExternArgs::new();
        ea.parse_response(&resp)?;

        let sfx = ea.get_args();
        assert!(sfx.len() > 0);

        Ok(())
    }
}
