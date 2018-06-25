// build.rs

use std::env;
use std::path::{Path, PathBuf};
#[macro_use]
extern crate error_chain;

#[cfg(windows)]
mod execs {
    use std::ffi::OsStr;
    use std::process::Command;

    pub fn cmd(program: &OsStr) -> Command {
        let mut cmd = Command::new("cmd");
        cmd.arg("/c");
        cmd.arg(program);
        cmd
    }
}
#[cfg(not(windows))]
mod execs {
    use std::ffi::OsStr;
    use std::process::Command;

    pub fn cmd(program: &OsStr) -> Command {
        Command::new(program)
    }
}

error_chain!{
 foreign_links {
        Io(std::io::Error);
    }
}

fn node_modules_exists() -> Result<()> {
    if Path::new("node_modules").exists() {
        Ok(())
    } else {
        bail!("`node_modules` does not exist. Please run `yarn install`")
    }
}

fn run() -> Result<()> {
    if let Ok(_) = env::var("CARGO_FEATURE_REGENERATE_CSS") {
        node_modules_exists()?;
        // Compile stylus stylesheet to css
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .chain_err(|| "Please run the script with: 'cargo build'!")?;
        let theme_dir = Path::new(&manifest_dir).join("src/theme/");
        let stylus_dir = theme_dir.join("stylus/book.styl");

        let stylus_path: PathBuf = [".", "node_modules", ".bin", "stylus"].iter().collect();

        if !execs::cmd(stylus_path.as_os_str())
            .arg(stylus_dir)
            .arg("--out")
            .arg(theme_dir)
            .arg("--use")
            .arg("nib")
            .status()?
            .success()
        {
            bail!("Stylus encountered an error");
        }
    }
    Ok(())
}

quick_main!(run);
