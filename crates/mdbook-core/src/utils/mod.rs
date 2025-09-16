//! Various helpers and utilities.

use anyhow::Error;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use tracing::error;

pub mod fs;
mod string;
mod toml_ext;

pub(crate) use self::toml_ext::TomlExt;

pub use self::string::{
    take_anchored_lines, take_lines, take_rustdoc_include_anchored_lines,
    take_rustdoc_include_lines,
};

/// Defines a `static` with a [`regex::Regex`].
#[macro_export]
macro_rules! static_regex {
    ($name:ident, $regex:literal) => {
        static $name: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new($regex).unwrap());
    };
    ($name:ident, bytes, $regex:literal) => {
        static $name: std::sync::LazyLock<regex::bytes::Regex> =
            std::sync::LazyLock::new(|| regex::bytes::Regex::new($regex).unwrap());
    };
}

/// Replaces multiple consecutive whitespace characters with a single space character.
pub fn collapse_whitespace(text: &str) -> Cow<'_, str> {
    static_regex!(WS, r"\s\s+");
    WS.replace_all(text, " ")
}

/// Convert the given string to a valid HTML element ID.
/// The only restriction is that the ID must not contain any ASCII whitespace.
pub fn normalize_id(content: &str) -> String {
    content
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
        .collect::<String>()
}

/// Generate an ID for use with anchors which is derived from a "normalised"
/// string.
pub fn id_from_content(content: &str) -> String {
    let mut content = content.to_string();

    // Skip any tags or html-encoded stuff
    static_regex!(HTML, r"(<.*?>)");
    content = HTML.replace_all(&content, "").into();
    const REPL_SUB: &[&str] = &["&lt;", "&gt;", "&amp;", "&#39;", "&quot;"];
    for sub in REPL_SUB {
        content = content.replace(sub, "");
    }

    // Remove spaces and hashes indicating a header
    let trimmed = content.trim().trim_start_matches('#').trim();
    normalize_id(trimmed)
}

/// Generate an ID for use with anchors which is derived from a "normalised"
/// string.
///
/// Each ID returned will be unique, if the same `id_counter` is provided on
/// each call.
pub fn unique_id_from_content(content: &str, id_counter: &mut HashMap<String, usize>) -> String {
    let id = id_from_content(content);

    // If we have headers with the same normalized id, append an incrementing counter
    let id_count = id_counter.entry(id.clone()).or_insert(0);
    let unique_id = match *id_count {
        0 => id,
        id_count => format!("{id}-{id_count}"),
    };
    *id_count += 1;
    unique_id
}

/// Prints a "backtrace" of some `Error`.
pub fn log_backtrace(e: &Error) {
    let mut message = format!("{e}");

    for cause in e.chain().skip(1) {
        write!(message, "\n\tCaused by: {cause}").unwrap();
    }

    error!("{message}");
}

/// Escape `<` and `>` for HTML.
pub fn bracket_escape(mut s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    let needs_escape: &[char] = &['<', '>'];
    while let Some(next) = s.find(needs_escape) {
        escaped.push_str(&s[..next]);
        match s.as_bytes()[next] {
            b'<' => escaped.push_str("&lt;"),
            b'>' => escaped.push_str("&gt;"),
            _ => unreachable!(),
        }
        s = &s[next + 1..];
    }
    escaped.push_str(s);
    escaped
}

#[cfg(test)]
mod tests {
    use super::bracket_escape;

    #[allow(deprecated)]
    mod id_from_content {
        use super::super::id_from_content;

        #[test]
        fn it_generates_anchors() {
            assert_eq!(
                id_from_content("## Method-call expressions"),
                "method-call-expressions"
            );
            assert_eq!(id_from_content("## **Bold** title"), "bold-title");
            assert_eq!(id_from_content("## `Code` title"), "code-title");
            assert_eq!(
                id_from_content("## title <span dir=rtl>foo</span>"),
                "title-foo"
            );
        }

        #[test]
        fn it_generates_anchors_from_non_ascii_initial() {
            assert_eq!(
                id_from_content("## `--passes`: add more rustdoc passes"),
                "--passes-add-more-rustdoc-passes"
            );
            assert_eq!(
                id_from_content("## 中文標題 CJK title"),
                "中文標題-cjk-title"
            );
            assert_eq!(id_from_content("## Über"), "Über");
        }
    }

    mod html_munging {
        use super::super::{normalize_id, unique_id_from_content};

        #[test]
        fn it_normalizes_ids() {
            assert_eq!(
                normalize_id("`--passes`: add more rustdoc passes"),
                "--passes-add-more-rustdoc-passes"
            );
            assert_eq!(
                normalize_id("Method-call 🐙 expressions \u{1f47c}"),
                "method-call--expressions-"
            );
            assert_eq!(normalize_id("_-_12345"), "_-_12345");
            assert_eq!(normalize_id("12345"), "12345");
            assert_eq!(normalize_id("中文"), "中文");
            assert_eq!(normalize_id("にほんご"), "にほんご");
            assert_eq!(normalize_id("한국어"), "한국어");
            assert_eq!(normalize_id(""), "");
        }

        #[test]
        fn it_generates_unique_ids_from_content() {
            // Same id if not given shared state
            assert_eq!(
                unique_id_from_content("## 中文標題 CJK title", &mut Default::default()),
                "中文標題-cjk-title"
            );
            assert_eq!(
                unique_id_from_content("## 中文標題 CJK title", &mut Default::default()),
                "中文標題-cjk-title"
            );

            // Different id if given shared state
            let mut id_counter = Default::default();
            assert_eq!(unique_id_from_content("## Über", &mut id_counter), "Über");
            assert_eq!(
                unique_id_from_content("## 中文標題 CJK title", &mut id_counter),
                "中文標題-cjk-title"
            );
            assert_eq!(unique_id_from_content("## Über", &mut id_counter), "Über-1");
            assert_eq!(unique_id_from_content("## Über", &mut id_counter), "Über-2");
        }
    }

    #[test]
    fn escaped_brackets() {
        assert_eq!(bracket_escape(""), "");
        assert_eq!(bracket_escape("<"), "&lt;");
        assert_eq!(bracket_escape(">"), "&gt;");
        assert_eq!(bracket_escape("<>"), "&lt;&gt;");
        assert_eq!(bracket_escape("<test>"), "&lt;test&gt;");
        assert_eq!(bracket_escape("a<test>b"), "a&lt;test&gt;b");
        assert_eq!(bracket_escape("'"), "'");
        assert_eq!(bracket_escape("\\"), "\\");
    }
}
