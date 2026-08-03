//! Preprocessor for the mdBook guide.

use mdbook_preprocessor::book::Book;
use mdbook_preprocessor::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use semver::{Version, VersionReq};
use std::io;

/// Preprocessing entry point.
pub fn handle_preprocessing() -> Result<()> {
    let pre = GuideHelper;
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook_preprocessor::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

struct GuideHelper;

impl Preprocessor for GuideHelper {
    fn name(&self) -> &str {
        "guide-helper"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        insert_version(&mut book);
        Ok(book)
    }
}

fn insert_version(book: &mut Book) {
    let path = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("Cargo.toml");
    let manifest_contents = std::fs::read_to_string(&path).unwrap();
    let manifest: toml::Value = toml::from_str(&manifest_contents).unwrap();
    let version = Version::parse(manifest["package"]["version"].as_str().unwrap())
        .expect("version is parseable");
    const MARKER: &str = "{{ mdbook-version }}";
    const SEMVER_MARKER: &str = "{{ mdbook-semver }}";
    const SEMVER_BREAK: &str = "{{ mdbook-semver-break }}";
    book.for_each_chapter_mut(|ch| {
        if ch.content.contains(MARKER) {
            ch.content = ch.content.replace(MARKER, &version.to_string());
        }
        if ch.content.contains(SEMVER_MARKER) {
            let semver_version = if version.major == 0 {
                format!("{}.{}", version.major, version.minor)
            } else {
                format!("{}", version.major)
            };
            ch.content = ch.content.replace(SEMVER_MARKER, &semver_version)
        }
        if ch.content.contains(SEMVER_BREAK) {
            let semver_break = if version.major == 0 {
                format!("{}.{}.0", version.major, version.minor + 1)
            } else {
                format!("{}.0.0", version.major + 1)
            };
            ch.content = ch.content.replace(SEMVER_BREAK, &semver_break);
        }
    });
}
