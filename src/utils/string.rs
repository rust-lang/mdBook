use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

/// Indication of whether to shift included text.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Shift {
    None,
    Left(usize),
    Right(usize),
}

fn shift_line(l: &str, shift: Shift) -> Cow<'_, str> {
    match shift {
        Shift::None => Cow::Borrowed(l),
        Shift::Right(shift) => {
            let indent = " ".repeat(shift);
            Cow::Owned(format!("{indent}{l}"))
        }
        Shift::Left(skip) => {
            if l.chars().take(skip).any(|c| !c.is_whitespace()) {
                log::error!("left-shifting away non-whitespace");
            }
            let rest = l.chars().skip(skip).collect::<String>();
            Cow::Owned(rest)
        }
    }
}

fn shift_lines(lines: &[String], shift: Shift) -> Vec<Cow<'_, str>> {
    lines.iter().map(|l| shift_line(l, shift)).collect()
}

/// Take a range of lines from a string.
pub fn take_lines<R: RangeBounds<usize>>(s: &str, range: R) -> String {
    take_lines_with_shift(s, range, Shift::None)
}

/// Take a range of lines from a string, shifting all lines left or right.
pub fn take_lines_with_shift<R: RangeBounds<usize>>(s: &str, range: R, shift: Shift) -> String {
    let start = match range.start_bound() {
        Excluded(&n) => n + 1,
        Included(&n) => n,
        Unbounded => 0,
    };
    let lines = s.lines().skip(start);
    let retained = match range.end_bound() {
        Excluded(end) => lines
            .take(end.saturating_sub(start))
            .map(|l| l.to_string())
            .collect::<Vec<_>>(),
        Included(end) => lines
            .take((end + 1).saturating_sub(start))
            .map(|l| l.to_string())
            .collect::<Vec<_>>(),
        Unbounded => lines.map(|l| l.to_string()).collect::<Vec<_>>(),
    };
    shift_lines(&retained, shift).join("\n")
}

static ANCHOR_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"ANCHOR:\s*(?P<anchor_name>[\w_-]+)").unwrap());
static ANCHOR_END: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"ANCHOR_END:\s*(?P<anchor_name>[\w_-]+)").unwrap());

/// Take anchored lines from a string.
/// Lines containing anchor are ignored.
pub fn take_anchored_lines(s: &str, anchor: &str) -> String {
    take_anchored_lines_with_shift(s, anchor, Shift::None)
}

/// Take anchored lines from a string, shifting all lines left or right.
/// Lines containing anchor are ignored.
pub fn take_anchored_lines_with_shift(s: &str, anchor: &str, shift: Shift) -> String {
    let mut retained = Vec::<String>::new();
    let mut anchor_found = false;

    for l in s.lines() {
        if anchor_found {
            match ANCHOR_END.captures(l) {
                Some(cap) => {
                    if &cap["anchor_name"] == anchor {
                        break;
                    }
                }
                None => {
                    if !ANCHOR_START.is_match(l) {
                        retained.push(l.to_string());
                    }
                }
            }
        } else if let Some(cap) = ANCHOR_START.captures(l) {
            if &cap["anchor_name"] == anchor {
                anchor_found = true;
            }
        }
    }

    shift_lines(&retained, shift).join("\n")
}

/// Keep lines contained within the range specified as-is.
/// For any lines not in the range, include them but use `#` at the beginning. This will hide the
/// lines from initial display but include them when expanding the code snippet or testing with
/// rustdoc.
pub fn take_rustdoc_include_lines<R: RangeBounds<usize>>(s: &str, range: R) -> String {
    let mut output = String::with_capacity(s.len());

    for (index, line) in s.lines().enumerate() {
        if !range.contains(&index) {
            output.push_str("# ");
        }
        output.push_str(line);
        output.push('\n');
    }
    output.pop();
    output
}

/// Keep lines between the anchor comments specified as-is.
/// For any lines not between the anchors, include them but use `#` at the beginning. This will
/// hide the lines from initial display but include them when expanding the code snippet or testing
/// with rustdoc.
pub fn take_rustdoc_include_anchored_lines(s: &str, anchor: &str) -> String {
    let mut output = String::with_capacity(s.len());
    let mut within_anchored_section = false;

    for l in s.lines() {
        if within_anchored_section {
            match ANCHOR_END.captures(l) {
                Some(cap) => {
                    if &cap["anchor_name"] == anchor {
                        within_anchored_section = false;
                    }
                }
                None => {
                    if !ANCHOR_START.is_match(l) {
                        output.push_str(l);
                        output.push('\n');
                    }
                }
            }
        } else if let Some(cap) = ANCHOR_START.captures(l) {
            if &cap["anchor_name"] == anchor {
                within_anchored_section = true;
            }
        } else if !ANCHOR_END.is_match(l) {
            output.push_str("# ");
            output.push_str(l);
            output.push('\n');
        }
    }

    output.pop();
    output
}

#[cfg(test)]
mod tests {
    use super::{
        shift_line, take_anchored_lines, take_anchored_lines_with_shift, take_lines,
        take_lines_with_shift, take_rustdoc_include_anchored_lines, take_rustdoc_include_lines,
        Shift,
    };

    #[test]
    fn shift_line_test() {
        let s = "    Line with 4 space intro";
        assert_eq!(shift_line(s, Shift::None), s);
        assert_eq!(shift_line(s, Shift::Left(4)), "Line with 4 space intro");
        assert_eq!(shift_line(s, Shift::Left(2)), "  Line with 4 space intro");
        assert_eq!(shift_line(s, Shift::Left(6)), "ne with 4 space intro");
        assert_eq!(
            shift_line(s, Shift::Right(2)),
            "      Line with 4 space intro"
        );
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)] // Intentionally checking that those are correctly handled
    fn take_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(take_lines(s, 1..3), "ipsum\ndolor");
        assert_eq!(take_lines(s, 3..), "sit\namet");
        assert_eq!(take_lines(s, ..3), "Lorem\nipsum\ndolor");
        assert_eq!(take_lines(s, ..), s);
        // corner cases
        assert_eq!(take_lines(s, 4..3), "");
        assert_eq!(take_lines(s, ..100), s);
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)] // Intentionally checking that those are correctly handled
    fn take_lines_with_shift_test() {
        let s = "  Lorem\n  ipsum\n    dolor\n  sit\n  amet";
        assert_eq!(
            take_lines_with_shift(s, 1..3, Shift::None),
            "  ipsum\n    dolor"
        );
        assert_eq!(
            take_lines_with_shift(s, 1..3, Shift::Left(2)),
            "ipsum\n  dolor"
        );
        assert_eq!(
            take_lines_with_shift(s, 1..3, Shift::Right(2)),
            "    ipsum\n      dolor"
        );
        assert_eq!(take_lines_with_shift(s, 3.., Shift::None), "  sit\n  amet");
        assert_eq!(
            take_lines_with_shift(s, 3.., Shift::Right(1)),
            "   sit\n   amet"
        );
        assert_eq!(take_lines_with_shift(s, 3.., Shift::Left(1)), " sit\n amet");
        assert_eq!(
            take_lines_with_shift(s, ..3, Shift::None),
            "  Lorem\n  ipsum\n    dolor"
        );
        assert_eq!(
            take_lines_with_shift(s, ..3, Shift::Right(4)),
            "      Lorem\n      ipsum\n        dolor"
        );
        assert_eq!(
            take_lines_with_shift(s, ..3, Shift::Left(4)),
            "rem\nsum\ndolor"
        );
        assert_eq!(take_lines_with_shift(s, .., Shift::None), s);
        // corner cases
        assert_eq!(take_lines_with_shift(s, 4..3, Shift::None), "");
        assert_eq!(take_lines_with_shift(s, 4..3, Shift::Left(2)), "");
        assert_eq!(take_lines_with_shift(s, 4..3, Shift::Right(2)), "");
        assert_eq!(take_lines_with_shift(s, ..100, Shift::None), s);
        assert_eq!(
            take_lines_with_shift(s, ..100, Shift::Right(2)),
            "    Lorem\n    ipsum\n      dolor\n    sit\n    amet"
        );
        assert_eq!(
            take_lines_with_shift(s, ..100, Shift::Left(2)),
            "Lorem\nipsum\n  dolor\nsit\namet"
        );
    }

    #[test]
    fn take_anchored_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(take_anchored_lines(s, "test"), "");

        let s = "Lorem\nipsum\ndolor\nANCHOR_END: test\nsit\namet";
        assert_eq!(take_anchored_lines(s, "test"), "");

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet";
        assert_eq!(take_anchored_lines(s, "test"), "dolor\nsit\namet");
        assert_eq!(take_anchored_lines(s, "something"), "");

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(take_anchored_lines(s, "test"), "dolor\nsit\namet");
        assert_eq!(take_anchored_lines(s, "something"), "");

        let s = "Lorem\nANCHOR: test\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(take_anchored_lines(s, "test"), "ipsum\ndolor\nsit\namet");
        assert_eq!(take_anchored_lines(s, "something"), "");

        let s = "Lorem\nANCHOR:    test2\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nANCHOR_END:test2\nipsum";
        assert_eq!(
            take_anchored_lines(s, "test2"),
            "ipsum\ndolor\nsit\namet\nlorem"
        );
        assert_eq!(take_anchored_lines(s, "test"), "dolor\nsit\namet");
        assert_eq!(take_anchored_lines(s, "something"), "");
    }

    #[test]
    fn take_anchored_lines_with_shift_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(take_anchored_lines_with_shift(s, "test", Shift::None), "");
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Right(2)),
            ""
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(2)),
            ""
        );

        let s = "Lorem\nipsum\ndolor\nANCHOR_END: test\nsit\namet";
        assert_eq!(take_anchored_lines_with_shift(s, "test", Shift::None), "");
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Right(2)),
            ""
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(2)),
            ""
        );

        let s = "  Lorem\n  ipsum\n  ANCHOR: test\n  dolor\n  sit\n  amet";
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::None),
            "  dolor\n  sit\n  amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Right(2)),
            "    dolor\n    sit\n    amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(2)),
            "dolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::None),
            ""
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::Right(2)),
            ""
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::Left(2)),
            ""
        );

        let s = "  Lorem\n  ipsum\n  ANCHOR: test\n  dolor\n  sit\n  amet\n  ANCHOR_END: test\n  lorem\n  ipsum";
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::None),
            "  dolor\n  sit\n  amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Right(2)),
            "    dolor\n    sit\n    amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(2)),
            "dolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(4)),
            "lor\nt\net"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(44)),
            "\n\n"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::None),
            ""
        );

        let s = "  Lorem\n  ANCHOR: test\n  ipsum\n  ANCHOR: test\n  dolor\n  sit\n  amet\n  ANCHOR_END: test\n  lorem\n  ipsum";
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::None),
            "  ipsum\n  dolor\n  sit\n  amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Right(2)),
            "    ipsum\n    dolor\n    sit\n    amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(2)),
            "ipsum\ndolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::None),
            ""
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::Right(2)),
            ""
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::Left(2)),
            ""
        );

        // Include non-ASCII.
        let s = "  Lorem\n  ANCHOR:    test2\n  ípsum\n  ANCHOR: test\n  dôlor\n  sit\n  amet\n  ANCHOR_END: test\n  lorem\n  ANCHOR_END:test2\n  ipsum";
        assert_eq!(
            take_anchored_lines_with_shift(s, "test2", Shift::None),
            "  ípsum\n  dôlor\n  sit\n  amet\n  lorem"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test2", Shift::Right(2)),
            "    ípsum\n    dôlor\n    sit\n    amet\n    lorem"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test2", Shift::Left(2)),
            "ípsum\ndôlor\nsit\namet\nlorem"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test2", Shift::Left(4)),
            "sum\nlor\nt\net\nrem"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::None),
            "  dôlor\n  sit\n  amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Right(2)),
            "    dôlor\n    sit\n    amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(2)),
            "dôlor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::None),
            ""
        );
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)] // Intentionally checking that those are correctly handled
    fn take_rustdoc_include_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(
            take_rustdoc_include_lines(s, 1..3),
            "# Lorem\nipsum\ndolor\n# sit\n# amet"
        );
        assert_eq!(
            take_rustdoc_include_lines(s, 3..),
            "# Lorem\n# ipsum\n# dolor\nsit\namet"
        );
        assert_eq!(
            take_rustdoc_include_lines(s, ..3),
            "Lorem\nipsum\ndolor\n# sit\n# amet"
        );
        assert_eq!(take_rustdoc_include_lines(s, ..), s);
        // corner cases
        assert_eq!(
            take_rustdoc_include_lines(s, 4..3),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet"
        );
        assert_eq!(take_rustdoc_include_lines(s, ..100), s);
    }

    #[test]
    fn take_rustdoc_include_anchored_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "test"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet"
        );

        let s = "Lorem\nipsum\ndolor\nANCHOR_END: test\nsit\namet";
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "test"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet"
        );

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet";
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "test"),
            "# Lorem\n# ipsum\ndolor\nsit\namet"
        );
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "something"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet"
        );

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "test"),
            "# Lorem\n# ipsum\ndolor\nsit\namet\n# lorem\n# ipsum"
        );
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "something"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet\n# lorem\n# ipsum"
        );

        let s = "Lorem\nANCHOR: test\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "test"),
            "# Lorem\nipsum\ndolor\nsit\namet\n# lorem\n# ipsum"
        );
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "something"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet\n# lorem\n# ipsum"
        );

        let s = "Lorem\nANCHOR:    test2\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nANCHOR_END:test2\nipsum";
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "test2"),
            "# Lorem\nipsum\ndolor\nsit\namet\nlorem\n# ipsum"
        );
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "test"),
            "# Lorem\n# ipsum\ndolor\nsit\namet\n# lorem\n# ipsum"
        );
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "something"),
            "# Lorem\n# ipsum\n# dolor\n# sit\n# amet\n# lorem\n# ipsum"
        );

        let s = "Lorem\nANCHOR: test\nipsum\nANCHOR_END: test\ndolor\nANCHOR: test\nsit\nANCHOR_END: test\namet";
        assert_eq!(
            take_rustdoc_include_anchored_lines(s, "test"),
            "# Lorem\nipsum\n# dolor\nsit\n# amet"
        );
    }
}
