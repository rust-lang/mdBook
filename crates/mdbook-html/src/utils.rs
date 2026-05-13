//! Utilities for processing HTML.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// Utility function to normalize path elements like `..`.
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(Component::RootDir);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if ret.ends_with(Component::ParentDir) {
                    ret.push(Component::ParentDir);
                } else {
                    let popped = ret.pop();
                    if !popped && !ret.has_root() {
                        ret.push(Component::ParentDir);
                    }
                }
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

/// Helper trait for converting a [`Path`] to a string suitable for an HTML path.
pub(crate) trait ToUrlPath {
    fn to_url_path(&self) -> String;
}

impl ToUrlPath for Path {
    fn to_url_path(&self) -> String {
        // We're generally assuming that all paths we deal with are utf-8.
        // The replace here is to handle Windows paths.
        self.to_str().unwrap().replace('\\', "/")
    }
}

/// Make sure an HTML id is unique.
///
/// Keeps a set of all previously returned IDs; if the requested id is already
/// used, numeric suffixes (-1, -2, ...) are tried until an unused one is found.
pub(crate) fn unique_id(id: &str, used: &mut HashSet<String>) -> String {
    if used.insert(id.to_string()) {
        return id.to_string();
    }

    // This ID is already in use. Generate one that is not by appending a
    // numeric suffix.
    let mut counter: u32 = 1;
    loop {
        let candidate = format!("{id}-{counter}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        counter += 1;
    }
}

/// Generates an HTML id from the given text.
pub(crate) fn id_from_content(content: &str) -> String {
    // This is intended to be close to how header ID generation is done in
    // other sites and tools, but is not 100% the same. Not all sites and
    // tools use the same algorithm. See these for more information:
    //
    // - https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#section-links
    // - https://docs.gitlab.com/user/markdown/#heading-ids-and-links
    // - https://pandoc.org/MANUAL.html#extension-auto_identifiers
    // - https://kramdown.gettalong.org/converter/html#auto-ids
    // - https://docs.rs/comrak/latest/comrak/options/struct.Extension.html#structfield.header_ids
    content
        .trim()
        .to_lowercase()
        .chars()
        .filter_map(|ch| {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                Some(ch)
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

/// Converts a logical HTML path to a clean URL link string.
///
/// Index pages have their `index.html` stripped, keeping a trailing slash.
/// Non-index pages have their `.html` extension replaced by `/`.
///
/// # Examples
///
/// - `foo/bar.html` → `"foo/bar/"`
/// - `foo/index.html` → `"foo/"`
/// - `index.html` → `"./"`
/// - `bar.html` → `"bar/"`
pub(crate) fn clean_url_link_path(logical_html_path: &Path) -> String {
    let is_index = logical_html_path.file_stem() == Some(OsStr::new("index"));

    if is_index {
        match logical_html_path.parent() {
            Some(parent) if parent.as_os_str().is_empty() => "./".to_string(),
            Some(parent) => format!("{}/", parent.to_url_path()),
            None => "./".to_string(),
        }
    } else {
        let without_ext = logical_html_path.with_extension("");
        format!("{}/", without_ext.to_url_path())
    }
}

/// Converts a logical HTML path to the physical output path in clean URL mode.
///
/// Index pages are left unchanged. Non-index pages are moved into a
/// subdirectory so the URL has no extension.
///
/// # Examples
///
/// - `foo/bar.html` → `PathBuf::from("foo/bar/index.html")`
/// - `foo/index.html` → `PathBuf::from("foo/index.html")`
/// - `index.html` → `PathBuf::from("index.html")`
/// - `bar.html` → `PathBuf::from("bar/index.html")`
pub(crate) fn clean_url_output_path(logical_html_path: &Path) -> PathBuf {
    let is_index = logical_html_path.file_stem() == Some(OsStr::new("index"));

    if is_index {
        logical_html_path.to_path_buf()
    } else {
        logical_html_path.with_extension("").join("index.html")
    }
}

/// Computes the path-to-root for a source `.md` file in clean URL mode.
///
/// Wraps [`mdbook_core::utils::fs::path_to_root`], adding one extra `../`
/// for non-index files because those pages are served from a subdirectory.
///
/// # Examples
///
/// - `foo/bar.md` → `"../../"`
/// - `foo/index.md` → `"../"`
/// - `index.md` → `""`
/// - `bar.md` → `"../"`
/// - `a/b/c.md` → `"../../../"`
pub(crate) fn clean_url_path_to_root(source_md_path: &Path) -> String {
    let is_index = source_md_path.file_stem() == Some(OsStr::new("index"));
    let mut root = mdbook_core::utils::fs::path_to_root(source_md_path);
    if !is_index {
        root.push_str("../");
    }
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_generates_unique_ids() {
        let mut id_counter = Default::default();

        assert_eq!(unique_id("", &mut id_counter), "");
        assert_eq!(unique_id("Über", &mut id_counter), "Über");
        assert_eq!(unique_id("Über", &mut id_counter), "Über-1");
        assert_eq!(unique_id("Über", &mut id_counter), "Über-2");
    }

    #[test]
    fn it_normalizes_ids() {
        assert_eq!(
            id_from_content("`--passes`: add more rustdoc passes"),
            "--passes-add-more-rustdoc-passes"
        );
        assert_eq!(
            id_from_content("Method-call 🐙 expressions \u{1f47c}"),
            "method-call--expressions-"
        );
        assert_eq!(id_from_content("_-_12345"), "_-_12345");
        assert_eq!(id_from_content("12345"), "12345");
        assert_eq!(id_from_content("中文"), "中文");
        assert_eq!(id_from_content("にほんご"), "にほんご");
        assert_eq!(id_from_content("한국어"), "한국어");
        assert_eq!(id_from_content(""), "");
        assert_eq!(id_from_content("中文標題 CJK title"), "中文標題-cjk-title");
        assert_eq!(id_from_content("Über"), "über");
    }

    #[test]
    fn clean_url_link_path_root_index() {
        assert_eq!(clean_url_link_path(Path::new("index.html")), "./");
    }

    #[test]
    fn clean_url_link_path_root_non_index() {
        assert_eq!(clean_url_link_path(Path::new("bar.html")), "bar/");
    }

    #[test]
    fn clean_url_link_path_nested_index() {
        assert_eq!(clean_url_link_path(Path::new("foo/index.html")), "foo/");
    }

    #[test]
    fn clean_url_link_path_nested_non_index() {
        assert_eq!(clean_url_link_path(Path::new("foo/bar.html")), "foo/bar/");
    }

    #[test]
    fn clean_url_link_path_deeply_nested() {
        assert_eq!(clean_url_link_path(Path::new("a/b/c.html")), "a/b/c/");
        assert_eq!(clean_url_link_path(Path::new("a/b/index.html")), "a/b/");
    }

    #[test]
    fn clean_url_output_path_root_index() {
        assert_eq!(
            clean_url_output_path(Path::new("index.html")),
            PathBuf::from("index.html")
        );
    }

    #[test]
    fn clean_url_output_path_root_non_index() {
        assert_eq!(
            clean_url_output_path(Path::new("bar.html")),
            PathBuf::from("bar/index.html")
        );
    }

    #[test]
    fn clean_url_output_path_nested_index() {
        assert_eq!(
            clean_url_output_path(Path::new("foo/index.html")),
            PathBuf::from("foo/index.html")
        );
    }

    #[test]
    fn clean_url_output_path_nested_non_index() {
        assert_eq!(
            clean_url_output_path(Path::new("foo/bar.html")),
            PathBuf::from("foo/bar/index.html")
        );
    }

    #[test]
    fn clean_url_output_path_deeply_nested() {
        assert_eq!(
            clean_url_output_path(Path::new("a/b/c.html")),
            PathBuf::from("a/b/c/index.html")
        );
        assert_eq!(
            clean_url_output_path(Path::new("a/b/index.html")),
            PathBuf::from("a/b/index.html")
        );
    }

    #[test]
    fn clean_url_path_to_root_root_index() {
        // index.md at root: path_to_root = "", no extra since index
        assert_eq!(clean_url_path_to_root(Path::new("index.md")), "");
    }

    #[test]
    fn clean_url_path_to_root_root_non_index() {
        // bar.md at root: path_to_root = "", +"../" since non-index
        assert_eq!(clean_url_path_to_root(Path::new("bar.md")), "../");
    }

    #[test]
    fn clean_url_path_to_root_nested_index() {
        // foo/index.md: path_to_root = "../", no extra since index
        assert_eq!(clean_url_path_to_root(Path::new("foo/index.md")), "../");
    }

    #[test]
    fn clean_url_path_to_root_nested_non_index() {
        // foo/bar.md: path_to_root = "../", +"../" since non-index
        assert_eq!(clean_url_path_to_root(Path::new("foo/bar.md")), "../../");
    }

    #[test]
    fn clean_url_path_to_root_deeply_nested() {
        // a/b/c.md: path_to_root(parent=a/b) = "../../", +"../" since non-index
        assert_eq!(clean_url_path_to_root(Path::new("a/b/c.md")), "../../../");
        // a/b/index.md: path_to_root(parent=a/b) = "../../", no extra since index
        assert_eq!(clean_url_path_to_root(Path::new("a/b/index.md")), "../../");
        // a/b/c/d.md: path_to_root(parent=a/b/c) = "../../../", +"../" since non-index
        assert_eq!(
            clean_url_path_to_root(Path::new("a/b/c/d.md")),
            "../../../../"
        );
    }
}
