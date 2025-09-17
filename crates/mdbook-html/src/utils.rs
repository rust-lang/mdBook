//! Utilities for processing HTML.

use std::collections::HashSet;
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
    content
        .trim()
        .chars()
        .filter_map(|ch| {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
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
        assert_eq!(id_from_content("Über"), "Über");
    }
}
