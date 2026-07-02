use mdbook_core::static_regex;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

/// Take a range of lines from a string.
pub(super) fn take_lines<R: RangeBounds<usize>>(s: &str, range: R) -> impl Iterator<Item = &str> {
    let start = match range.start_bound() {
        Excluded(&n) => n + 1,
        Included(&n) => n,
        Unbounded => 0,
    };
    let lines = s.lines().skip(start);
    match range.end_bound() {
        Excluded(end) => lines.take(end.saturating_sub(start)),
        Included(end) => lines.take((end + 1).saturating_sub(start)),
        Unbounded => lines.take(usize::MAX),
    }
}

static_regex!(ANCHOR_START, r"ANCHOR:\s*(?P<anchor_name>[\w_-]+)");
static_regex!(ANCHOR_END, r"ANCHOR_END:\s*(?P<anchor_name>[\w_-]+)");

/// Take anchored lines from a string.
/// Lines containing anchor are ignored.
pub(super) fn take_anchored_lines<'a>(s: &'a str, anchor: &str) -> impl Iterator<Item = &'a str> {
    let mut in_anchor = false;
    s.lines().filter(move |line| {
        if in_anchor {
            if let Some(captures) = ANCHOR_END.captures(line) {
                if captures[1] == *anchor {
                    in_anchor = false;
                }
                return false;
            }
            return !ANCHOR_START.is_match(line);
        }

        if ANCHOR_END.is_match(line) {
            return false;
        }

        if let Some(captures) = ANCHOR_START.captures(line) {
            if captures[1] == *anchor {
                in_anchor = true;
            }
        }

        false
    })
}

/// Returns an iterator over (line, true) for lines within the specified range,
/// and (line, false) for those outside of the range.
/// This is to allow hiding the lines from initial display but include them when
/// expanding the code snippet or testing with rustdoc.
pub(super) fn take_rustdoc_lines<R: RangeBounds<usize>>(
    s: &str,
    range: R,
) -> impl Iterator<Item = (&str, bool)> {
    s.lines()
        .enumerate()
        .map(move |(index, line)| (line, range.contains(&index)))
}

/// Returns an iterator over (line, true) for lines between the specified anchor
/// comments, and (line, false) for those outside of the specified anchor.
/// This is to allow hiding the lines from initial display but include them when
/// expanding the code snippet or testing with rustdoc.
pub(super) fn take_rustdoc_anchored_lines<'a>(
    s: &'a str,
    anchor: &str,
) -> impl Iterator<Item = (&'a str, bool)> {
    let mut in_anchor = false;
    s.lines().filter_map(move |line| {
        if in_anchor {
            if let Some(captures) = ANCHOR_END.captures(line) {
                if captures[1] == *anchor {
                    in_anchor = false;
                }
                return None;
            }
            if ANCHOR_START.is_match(line) {
                return None;
            }
            return Some((line, true));
        }

        if ANCHOR_END.is_match(line) {
            return None;
        }

        if let Some(captures) = ANCHOR_START.captures(line) {
            if captures[1] == *anchor {
                in_anchor = true;
            }
            return None;
        }

        Some((line, false))
    })
}

#[cfg(test)]
mod tests {
    use super::{take_anchored_lines, take_lines, take_rustdoc_anchored_lines, take_rustdoc_lines};

    #[test]
    #[allow(clippy::reversed_empty_ranges)] // Intentionally checking that those are correctly handled
    fn take_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(
            take_lines(s, 1..3).collect::<Vec<_>>().join("\n"),
            "ipsum\ndolor"
        );
        assert_eq!(
            take_lines(s, 3..).collect::<Vec<_>>().join("\n"),
            "sit\namet"
        );
        assert_eq!(
            take_lines(s, ..3).collect::<Vec<_>>().join("\n"),
            "Lorem\nipsum\ndolor"
        );
        assert_eq!(take_lines(s, ..).collect::<Vec<_>>().join("\n"), s);
        // corner cases
        assert_eq!(take_lines(s, 4..3).collect::<Vec<_>>().join("\n"), "");
        assert_eq!(take_lines(s, ..100).collect::<Vec<_>>().join("\n"), s);
    }

    #[test]
    fn take_anchored_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(
            take_anchored_lines(s, "test")
                .collect::<Vec<_>>()
                .join("\n"),
            ""
        );

        let s = "Lorem\nipsum\ndolor\nANCHOR_END: test\nsit\namet";
        assert_eq!(
            take_anchored_lines(s, "test")
                .collect::<Vec<_>>()
                .join("\n"),
            ""
        );

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet";
        assert_eq!(
            take_anchored_lines(s, "test")
                .collect::<Vec<_>>()
                .join("\n"),
            "dolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines(s, "something")
                .collect::<Vec<_>>()
                .join("\n"),
            ""
        );

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(
            take_anchored_lines(s, "test")
                .collect::<Vec<_>>()
                .join("\n"),
            "dolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines(s, "something")
                .collect::<Vec<_>>()
                .join("\n"),
            ""
        );

        let s = "Lorem\nANCHOR: test\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(
            take_anchored_lines(s, "test")
                .collect::<Vec<_>>()
                .join("\n"),
            "ipsum\ndolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines(s, "something")
                .collect::<Vec<_>>()
                .join("\n"),
            ""
        );

        let s = "Lorem\nANCHOR:    test2\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nANCHOR_END:test2\nipsum";
        assert_eq!(
            take_anchored_lines(s, "test2")
                .collect::<Vec<_>>()
                .join("\n"),
            "ipsum\ndolor\nsit\namet\nlorem"
        );
        assert_eq!(
            take_anchored_lines(s, "test")
                .collect::<Vec<_>>()
                .join("\n"),
            "dolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines(s, "something")
                .collect::<Vec<_>>()
                .join("\n"),
            ""
        );

        let s = "Lorem\nANCHOR: test\nipsum\nANCHOR_END: test\ndolor\nANCHOR: test\nsit\nANCHOR_END: test\namet";
        assert_eq!(
            take_anchored_lines(s, "test")
                .collect::<Vec<_>>()
                .join("\n"),
            "ipsum\nsit"
        );
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)] // Intentionally checking that those are correctly handled
    fn take_rustdoc_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(
            take_rustdoc_lines(s, 1..3)
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\nipsum\ndolor\n# sit\n# amet"
        );
        assert_eq!(
            take_rustdoc_lines(s, 3..)
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\n# dolor\nsit\namet"
        );
        assert_eq!(
            take_rustdoc_lines(s, ..3)
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "Lorem\nipsum\ndolor\n# sit\n# amet"
        );
        assert_eq!(
            take_rustdoc_lines(s, ..)
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            s
        );
        // corner cases
        assert_eq!(
            take_rustdoc_lines(s, 4..3)
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet"
        );
        assert_eq!(
            take_rustdoc_lines(s, ..100)
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            s
        );
    }

    #[test]
    fn take_rustdoc_anchored_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(
            take_rustdoc_anchored_lines(s, "test")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet"
        );

        let s = "Lorem\nipsum\ndolor\nANCHOR_END: test\nsit\namet";
        assert_eq!(
            take_rustdoc_anchored_lines(s, "test")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet"
        );

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet";
        assert_eq!(
            take_rustdoc_anchored_lines(s, "test")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\ndolor\nsit\namet"
        );
        assert_eq!(
            take_rustdoc_anchored_lines(s, "something")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet"
        );

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(
            take_rustdoc_anchored_lines(s, "test")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\ndolor\nsit\namet\n# lorem\n# ipsum"
        );
        assert_eq!(
            take_rustdoc_anchored_lines(s, "something")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet\n# lorem\n# ipsum"
        );

        let s = "Lorem\nANCHOR: test\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(
            take_rustdoc_anchored_lines(s, "test")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\nipsum\ndolor\nsit\namet\n# lorem\n# ipsum"
        );
        assert_eq!(
            take_rustdoc_anchored_lines(s, "something")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet\n# lorem\n# ipsum"
        );

        let s = "Lorem\nANCHOR:    test2\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nANCHOR_END:test2\nipsum";
        assert_eq!(
            take_rustdoc_anchored_lines(s, "test2")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\nipsum\ndolor\nsit\namet\nlorem\n# ipsum"
        );
        assert_eq!(
            take_rustdoc_anchored_lines(s, "test")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\ndolor\nsit\namet\n# lorem\n# ipsum"
        );
        assert_eq!(
            take_rustdoc_anchored_lines(s, "something")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet\n# lorem\n# ipsum"
        );

        let s = "Lorem\nANCHOR: test\nipsum\nANCHOR_END: test\ndolor\nANCHOR: test\nsit\nANCHOR_END: test\namet";
        assert_eq!(
            take_rustdoc_anchored_lines(s, "test")
                .map(|(line, show)| { format!("{}{line}", show.then_some("").unwrap_or("# ")) })
                .collect::<Vec<_>>()
                .join("\n"),
            "# Lorem\nipsum\n# dolor\nsit\n# amet"
        );
    }
}
