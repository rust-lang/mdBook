//! Get "compiler" args from cargo

use crate::errors::*;
use anyhow::anyhow;
use cargo_manifest::{Edition, Manifest, MaybeInherited::Local};
use log::{debug, info};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
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
/// let manifest_file = std::env::current_dir()?.join("Cargo.toml");    // or other path to `Cargo.toml`
/// let mut extern_args = ExternArgs::new();
/// extern_args.load(&manifest_file)?;
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
    /// rust edition as specified in manifest
    pub edition: String, // where default value of "" means arg wasn't specified
    /// crate name as specified in manifest
    pub crate_name: String,
    // accumulated library path(s), as observed from live cargo run
    lib_list: Vec<String>,
    // explicit extern crates, as observed from live cargo run
    extern_list: Vec<String>,
}

impl ExternArgs {
    /// simple constructor
    pub fn new() -> Self {
        ExternArgs {
            edition: String::default(),
            crate_name: String::default(),
            lib_list: vec![],
            extern_list: vec![],
        }
    }

    /// Run a `cargo build` to see what args Cargo is using for library paths and extern crates.
    /// Touch a source file in the crate to ensure something is compiled and the args will be visible.
    pub fn load(&mut self, cargo_path: &Path) -> Result<&Self> {
        // find Cargo.toml and determine the package name and lib or bin source file.
        let proj_root = cargo_path
            .canonicalize()
            .context(format!(
                "can't find cargo manifest {}",
                &cargo_path.to_string_lossy()
            ))?
            .parent()
            .ok_or(anyhow!("can't find parent of {:?}", cargo_path))?
            .to_owned();
        let mut manifest = Manifest::from_path(cargo_path).context(format!(
            "can't open cargo manifest {}",
            &cargo_path.to_string_lossy()
        ))?;
        manifest.complete_from_path(&proj_root)?; // try real hard to determine bin or lib
        let package = manifest
            .package
            .expect("doctest Cargo.toml must include a [package] section");

        self.crate_name = package.name.replace('-', "_"); // maybe cargo shouldn't allow packages to include non-identifier characters?
                                                          // in any case, this won't work when default crate doesn't have package name (which I'm sure cargo allows somehow or another)
        self.edition = if let Some(Local(edition)) = package.edition {
            my_display_edition(edition)
        } else {
            "".to_owned() //
        };

        debug!(
            "parsed from manifest: name: {}, edition: {}",
            self.crate_name,
            format!("{:?}", self.edition)
        );

        // touch (change) a file in the project to force check to do something
        // I haven't figured out how to determine bin or lib source file from cargo, fall back on heuristics here.

        for fname in ["main.rs", "lib.rs"] {
            let try_path: PathBuf = proj_root.join("src").join(fname);
            if try_path.exists() {
                touch(&try_path)?;
                self.run_cargo(&proj_root, cargo_path)?;
                return Ok(self);
                // file should be closed when f goes out of scope at bottom of this loop
            }
        }
        bail!("Couldn't find lib or bin source in project {:?}", proj_root)
    }

    fn run_cargo(&mut self, proj_root: &Path, manifest_path: &Path) -> Result<&Self> {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(proj_root)
            .arg("build")
            .arg("--verbose")
            .arg("--manifest-path")
            .arg(manifest_path);
        info!("running {:?}", cmd);

        let output = cmd.output()?;

        if !output.status.success() {
            bail!(
                "Exit status {} from {:?}\nMessage:\n{:?}",
                output.status,
                cmd,
                std::string::String::from_utf8_lossy(&output.stderr)
            );
        }

        //ultimatedebug std::fs::write(proj_root.join("mdbook_cargo_out.txt"), &output.stderr)?;

        let cmd_resp: &str = std::str::from_utf8(&output.stderr)?;
        self.parse_response(self.crate_name.clone().as_str(), cmd_resp)?;

        Ok(self)
    }

    /// Parse response stdout+stderr response from `cargo build`
    /// into arguments we can use to invoke rustdoc (--edition --extern and -L).
    /// The response may contain multiple builds, scan for the one that corresponds to the doctest crate.
    ///
    /// > This parser is broken, doesn't handle arg values with embedded spaces (single quoted).
    /// > Fortunately, the args we care about (so far) don't have those kinds of values.
    pub fn parse_response(&mut self, my_crate: &str, buf: &str) -> Result<()> {
        let mut builds_ignored = 0;

        let my_cn_arg = format!(" --crate-name {}", my_crate);
        for l in buf.lines() {
            if let Some(_i) = l.find(" Running ") {
                if let Some(_cn_pos) = l.find(&my_cn_arg) {
                    let args_seg: &str = l.split('`').skip(1).take(1).collect::<Vec<_>>()[0]; // sadly, cargo decorates string with backticks
                    let mut arg_iter = args_seg.split_whitespace();

                    while let Some(arg) = arg_iter.next() {
                        match arg {
                            "-L" | "--library-path" => {
                                self.lib_list
                                    .push(arg_iter.next().unwrap_or_default().to_owned());
                            }

                            "--extern" => {
                                let mut dep_arg = arg_iter.next().unwrap_or_default().to_owned();

                                // sometimes, build references the.rmeta even though our doctests will require .rlib
                                // so convert the argument and hope for the best.
                                // if .rlib is not there when the doctest runs, it will complain.
                                if dep_arg.ends_with(".rmeta") {
                                    debug!(
                                        "Build referenced {}, converted to .rlib hoping that actual file will be there in time.",
                                        dep_arg);
                                    dep_arg = dep_arg.replace(".rmeta", ".rlib");
                                }
                                self.extern_list.push(dep_arg);
                            }

                            "--crate-name" => {
                                self.crate_name = arg_iter.next().unwrap_or_default().to_owned();
                            }

                            _ => {
                                if let Some((kw, val)) = arg.split_once('=') {
                                    if kw == "--edition" {
                                        self.edition = val.to_owned();
                                    }
                                }
                            }
                        }
                    }
                } else {
                    builds_ignored += 1;
                }
            };
        }

        if self.extern_list.is_empty() || self.lib_list.is_empty() {
            bail!("Couldn't extract -L or --extern args from Cargo, is current directory == cargo project root?");
        }

        debug!(
            "Ignored {} other builds performed in this run",
            builds_ignored
        );

        Ok(())
    }

    /// provide the parsed external args used to invoke rustdoc (--edition, -L and --extern).
    pub fn get_args(&self) -> Vec<String> {
        let mut ret_val: Vec<String> = vec![];
        for i in &self.lib_list {
            ret_val.push("-L".to_owned());
            ret_val.push(i.clone());
        }
        for j in &self.extern_list {
            ret_val.push("--extern".to_owned());
            ret_val.push(j.clone());
        }
        ret_val
    }
}

impl Default for ExternArgs {
    fn default() -> Self {
        Self::new()
    }
}

fn my_display_edition(edition: Edition) -> String {
    match edition {
        Edition::E2015 => "2015",
        Edition::E2018 => "2018",
        Edition::E2021 => "2021",
        Edition::E2024 => "2024",
    }
    .to_owned()
}
// Private "touch" function to update file modification time without changing content.
// needed because [std::fs::set_modified] is unstable in rust 1.74,
// which is currently the MSRV for mdBook.  It is available in rust 1.76 onward.

fn touch(victim: &Path) -> Result<()> {
    let curr_content = fs::read(victim).with_context(|| "reading existing file")?;
    let mut touchfs = File::options()
        .append(true)
        .open(victim)
        .with_context(|| "opening for touch")?;

    let _len_written = touchfs.write(b"z")?; // write a byte
    touchfs.flush().expect("closing"); // close the file
    drop(touchfs); // close modified file, hopefully updating modification time

    fs::write(victim, curr_content).with_context(|| "trying to restore old content")
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile;

    #[test]
    fn parse_response_parses_string() -> Result<()> {
        let test_str = r###"
        Fresh unicode-ident v1.0.14
        Fresh cfg-if v1.0.0
        Fresh memchr v2.7.4
        Fresh autocfg v1.4.0
        Fresh version_check v0.9.5
        --- clip ---
        Fresh bytecount v0.6.8
        Fresh leptos_router v0.7.0
        Fresh leptos_meta v0.7.0
        Fresh console_error_panic_hook v0.1.7
        Fresh mdbook-keeper v0.5.0
        Dirty leptos-book v0.1.0 (/home/bobhy/src/localdep/book): the file `src/lib.rs` has changed (1733758773.052514835s, 10h 32m 29s after last build at 1733720824.458358565s)
    Compiling leptos-book v0.1.0 (/home/bobhy/src/localdep/book)
      Running `/home/bobhy/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc --crate-name leptos_book --edition=2021 src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type cdylib --crate-type rlib --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs)' --check-cfg 'cfg(feature, values("hydrate", "ssr"))' -C metadata=2eec49d479de095c --out-dir /home/bobhy/src/localdep/book/target/debug/deps -C incremental=/home/bobhy/src/localdep/book/target/debug/incremental -L dependency=/home/bobhy/src/localdep/book/target/debug/deps --extern console_error_panic_hook=/home/bobhy/src/localdep/book/target/debug/deps/libconsole_error_panic_hook-d34cf0116774f283.rlib --extern http=/home/bobhy/src/localdep/book/target/debug/deps/libhttp-d4d503240b7a6b18.rlib --extern leptos=/home/bobhy/src/localdep/book/target/debug/deps/libleptos-1dabf2e09ca58f3d.rlib --extern leptos_meta=/home/bobhy/src/localdep/book/target/debug/deps/libleptos_meta-df8ce1704acca063.rlib --extern leptos_router=/home/bobhy/src/localdep/book/target/debug/deps/libleptos_router-df109cd2ee44b2a0.rlib --extern mdbook_keeper_lib=/home/bobhy/src/localdep/book/target/debug/deps/libmdbook_keeper_lib-f4016aaf2c5da5f2.rlib --extern thiserror=/home/bobhy/src/localdep/book/target/debug/deps/libthiserror-acc5435cdf9551fe.rlib --extern wasm_bindgen=/home/bobhy/src/localdep/book/target/debug/deps/libwasm_bindgen-89a7b1dccd9668ae.rlib`
     Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.60s
 
     "###;

        let mut ea = ExternArgs::new();
        ea.parse_response("leptos_book", &test_str)?;

        let args = ea.get_args();

        assert_eq!(ea.edition, "2021");
        assert_eq!(ea.crate_name, "leptos_book");

        assert_eq!(18, args.len());

        assert_eq!(1, args.iter().filter(|i| *i == "-L").count());
        assert_eq!(8, args.iter().filter(|i| *i == "--extern").count());

        Ok(())
    }

    #[test]
    fn verify_touch() -> Result<()> {
        const FILE_CONTENT: &[u8] =
            b"I am some random text with crlfs \r\n but also nls \n and terminated with a nl \n";
        const DELAY: Duration = Duration::from_millis(20); // don't hang up tests  for too long, but maybe 10ms is too short?

        let temp_dir = tempfile::TempDir::new()?;
        let mut victim_path = temp_dir.path().to_owned();
        victim_path.push("workfile.dir");
        fs::write(&victim_path, FILE_CONTENT)?;
        let old_md = fs::metadata(&victim_path)?;
        thread::sleep(DELAY);

        touch(&victim_path)?;
        let new_md = fs::metadata(&victim_path)?;

        let act_content = fs::read(&victim_path)?;

        assert_eq!(FILE_CONTENT, act_content);
        let tdif = new_md
            .modified()
            .expect("getting modified time new")
            .duration_since(old_md.modified().expect("getting modified time old"))
            .expect("system time botch");
        // can't expect sleep 20ms to actually delay exactly that --
        // but the test is to verify that `touch` made the file look any newer.
        // Give ourselves 50% slop under what we were aiming for and call it good enough.
        assert!(
            tdif >= (DELAY / 2),
            "verify_touch: expected {:?}, actual {:?}",
            DELAY,
            tdif
        );
        Ok(())
    }
}
