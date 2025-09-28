//! Helper to generate a changelog for a new release.

use super::Result;
use std::fs;
use std::process::Command;
use std::process::exit;

const CHANGELOG_PATH: &str = "CHANGELOG.md";

pub(crate) fn changelog() -> Result<()> {
    let previous = get_previous()?;
    let current = get_current()?;
    if current == previous {
        eprintln!(
            "error: Current version is `{current}` which is the same as the \
             previous version in the changelog. Run `cargo set-version --bump <BUMP> first."
        );
        exit(1);
    }
    let prs = get_prs(&previous)?;
    update_changelog(&previous, &current, &prs)?;
    Ok(())
}

fn get_previous() -> Result<String> {
    let contents = fs::read_to_string(CHANGELOG_PATH)?;
    let version = contents
        .lines()
        .filter_map(|line| line.strip_prefix("## mdBook "))
        .next()
        .expect("at least one entry")
        .to_owned();
    Ok(version)
}

fn get_current() -> Result<String> {
    let contents = fs::read_to_string("Cargo.toml")?;
    let mut lines = contents
        .lines()
        .filter_map(|line| line.strip_prefix("version = "))
        .map(|version| &version[1..version.len() - 1]);
    let version = lines.next().expect("version should exist").to_owned();
    assert_eq!(lines.next(), None);
    Ok(version)
}

fn get_prs(previous: &str) -> Result<Vec<(String, String)>> {
    println!("running `git fetch upstream`");
    let status = Command::new("git").args(["fetch", "upstream"]).status()?;
    if !status.success() {
        eprintln!("error: git fetch failed");
        exit(1);
    }
    println!("running `git log`");
    const SEPARATOR: &str = "---COMMIT_SEPARATOR---";
    let output = Command::new("git")
        .args([
            "log",
            "--first-parent",
            &format!("--pretty=format:%B%n{SEPARATOR}"),
            "upstream/master",
            &format!("v{previous}...upstream/HEAD"),
        ])
        .output()?;
    if !output.status.success() {
        eprintln!("error: git log failed");
        exit(1);
    }
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let prs = stdout
        .split(&format!("{SEPARATOR}\n"))
        .filter_map(|entry| {
            let mut lines = entry.lines();
            let first = match lines.next().unwrap().strip_prefix("Merge pull request #") {
                Some(f) => f,
                None => {
                    println!("warning: merge line not found in {entry}");
                    return None;
                }
            };
            let number = first.split_whitespace().next().unwrap();
            assert_eq!(lines.next(), Some(""));
            let title = lines.next().expect("title is set");
            assert_eq!(lines.next(), Some(""));
            Some((number.to_string(), title.to_string()))
        })
        .collect();
    Ok(prs)
}

fn update_changelog(previous: &str, current: &str, prs: &[(String, String)]) -> Result<()> {
    let prs: String = prs
        .iter()
        .map(|(number, title)| {
            format!(
                "- {title}\n  \
                [#{number}](https://github.com/rust-lang/mdBook/pull/{number})\n"
            )
        })
        .collect();
    let new = format!(
        "## mdBook {current}\n\
        [v{previous}...v{current}](https://github.com/rust-lang/mdBook/compare/v{previous}...v{current})\n\
        \n\
        {prs}\
        \n\
        ### Added\n\
        \n\
        ### Changed\n\
        \n\
        ### Fixed\n\
        \n"
    );

    let mut contents = fs::read_to_string(CHANGELOG_PATH)?;
    let insertion_point = contents.find("## ").unwrap();
    contents.insert_str(insertion_point, &new);
    fs::write(CHANGELOG_PATH, contents)?;
    Ok(())
}
