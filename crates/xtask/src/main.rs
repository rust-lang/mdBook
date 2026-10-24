//! Helper for local development.

use std::collections::BTreeMap;
use std::error::Error;
use std::io::Write;
use std::process::exit;
use std::process::{Command, Stdio};

mod changelog;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    macro_rules! commands {
        ($($name:literal => $func:expr),* $(,)?) => {
            [$(($name, $func as fn() -> Result<()>)),*]
        };
    }

    let cmds: BTreeMap<&'static str, fn() -> Result<()>> = commands! {
        "test-all" => test_all,
        "test-workspace" => test_workspace,
        "clippy" => clippy,
        "doc" => doc,
        "fmt" => fmt,
        "semver-checks" => semver_checks,
        "eslint" => eslint,
        "gui" => gui,
        "changelog" => changelog::changelog,
    }
    .into_iter()
    .collect();
    let keys = cmds.keys().copied().collect::<Vec<_>>().join(", ");
    let mut args = std::env::args().skip(1).peekable();
    if args.peek().is_none() {
        eprintln!("error: specify a command (valid options: {keys})");
        exit(1);
    }
    while let Some(arg) = args.next() {
        if let Some(cmd_fn) = cmds.get(arg.as_str()) {
            cmd_fn()?;
        } else if arg == "bump" {
            let bump_arg = args
                .next()
                .expect("the next argument should be one of major, minor, patch, rc, beta, alpha");
            bump(&bump_arg)?;
        } else if matches!(arg.as_str(), "-h" | "--help") {
            println!("valid options: {keys}");
            exit(0)
        } else {
            eprintln!("error: unknown command `{arg}` (valid options: {keys}");
            exit(1);
        }
    }
    println!("success!");
    Ok(())
}

fn test_all() -> Result<()> {
    test_workspace()?;
    clippy()?;
    doc()?;
    fmt()?;
    semver_checks()?;
    eslint()?;
    gui()?;
    Ok(())
}

fn cargo(args: &str, cb: &dyn Fn(&mut Command)) -> Result<()> {
    println!("Running `cargo {args}`");
    let mut cmd = Command::new("cargo");
    cmd.args(args.split_whitespace());
    cb(&mut cmd);
    let status = cmd.status().expect("cargo should be installed");
    if !status.success() {
        return Err(format!("command `cargo {args}` failed").into());
    }
    Ok(())
}

fn test_workspace() -> Result<()> {
    cargo("test --workspace", &|_| {})?;
    cargo("test --workspace --no-default-features", &|_| {})?;
    Ok(())
}

fn clippy() -> Result<()> {
    cargo(
        "clippy --workspace --all-targets --no-deps -- -D warnings",
        &|_| {},
    )?;
    Ok(())
}

fn doc() -> Result<()> {
    cargo(
        "doc --workspace --document-private-items --no-deps",
        &|cmd| {
            cmd.env("RUSTDOCFLAGS", "-D warnings");
        },
    )?;
    Ok(())
}

fn fmt() -> Result<()> {
    cargo("fmt --check", &|_| {})?;
    Ok(())
}

fn semver_checks() -> Result<()> {
    cargo("+stable semver-checks --workspace", &|_| {})?;
    Ok(())
}

fn gui() -> Result<()> {
    cargo("test --test gui", &|_| {})?;
    Ok(())
}

fn eslint() -> Result<()> {
    println!("Running `npm run lint`");
    let status = Command::new("npm")
        .args(["run", "lint"])
        .status()
        .expect("npm should be installed");
    if !status.success() {
        return Err("eslint failed".into());
    }
    Ok(())
}

fn bump(bump: &str) -> Result<()> {
    // Grab all the publishable crate names.
    let metadata = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()?;
    let mut jq = Command::new("jq")
        .args(["-r", ".packages[] | select(.publish == null) | .name"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    jq.stdin.as_mut().unwrap().write_all(&metadata.stdout)?;
    let jq_out = jq.wait_with_output()?;
    if !jq_out.status.success() {
        eprintln!("jq failed");
        exit(1);
    }
    let names = std::str::from_utf8(&jq_out.stdout).unwrap();
    let mut names: Vec<_> = names.split_whitespace().collect();
    for i in (0..names.len()).rev() {
        names.insert(i, "-p");
    }

    let status = Command::new("cargo")
        .args(["set-version", "--bump"])
        .arg(bump)
        .args(names)
        .status()?;
    if !status.success() {
        eprintln!("cargo set-version failed");
        exit(1);
    }
    Ok(())
}
