//! Utilities for dealing with HTML.

use std::borrow::Cow;

/// Escape characters to make it safe for an HTML string.
pub fn escape_html_attribute(text: &str) -> Cow<'_, str> {
    let needs_escape: &[char] = &['<', '>', '\'', '"', '\\', '&'];
    let mut s = text;
    let mut output = String::new();
    while let Some(next) = s.find(needs_escape) {
        output.push_str(&s[..next]);
        match s.as_bytes()[next] {
            b'<' => output.push_str("&lt;"),
            b'>' => output.push_str("&gt;"),
            b'\'' => output.push_str("&#39;"),
            b'"' => output.push_str("&quot;"),
            b'\\' => output.push_str("&#92;"),
            b'&' => output.push_str("&amp;"),
            _ => unreachable!(),
        }
        s = &s[next + 1..];
    }
    if output.is_empty() {
        Cow::Borrowed(text)
    } else {
        output.push_str(s);
        Cow::Owned(output)
    }
}

/// Escape `<`, `>`, and '&' for HTML.
pub fn escape_html(text: &str) -> Cow<'_, str> {
    let needs_escape: &[char] = &['<', '>', '&'];
    let mut s = text;
    let mut output = String::new();
    while let Some(next) = s.find(needs_escape) {
        output.push_str(&s[..next]);
        match s.as_bytes()[next] {
            b'<' => output.push_str("&lt;"),
            b'>' => output.push_str("&gt;"),
            b'&' => output.push_str("&amp;"),
            _ => unreachable!(),
        }
        s = &s[next + 1..];
    }
    if output.is_empty() {
        Cow::Borrowed(text)
    } else {
        output.push_str(s);
        Cow::Owned(output)
    }
}

#[test]
fn attributes_are_escaped() {
    assert_eq!(escape_html_attribute(""), "");
    assert_eq!(escape_html_attribute("<"), "&lt;");
    assert_eq!(escape_html_attribute(">"), "&gt;");
    assert_eq!(escape_html_attribute("<>"), "&lt;&gt;");
    assert_eq!(escape_html_attribute("<test>"), "&lt;test&gt;");
    assert_eq!(escape_html_attribute("a<test>b"), "a&lt;test&gt;b");
    assert_eq!(escape_html_attribute("'"), "&#39;");
    assert_eq!(escape_html_attribute("\\"), "&#92;");
    assert_eq!(escape_html_attribute("&"), "&amp;");
}

#[test]
fn html_is_escaped() {
    assert_eq!(escape_html(""), "");
    assert_eq!(escape_html("<"), "&lt;");
    assert_eq!(escape_html(">"), "&gt;");
    assert_eq!(escape_html("&"), "&amp;");
    assert_eq!(escape_html("<>"), "&lt;&gt;");
    assert_eq!(escape_html("<test>"), "&lt;test&gt;");
    assert_eq!(escape_html("a<test>b"), "a&lt;test&gt;b");
    assert_eq!(escape_html("'"), "'");
    assert_eq!(escape_html("\\"), "\\");
}
