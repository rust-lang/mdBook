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
    /// Strip leftmost whitespace that is common to all lines.
    Auto,
}

/// Indication of how much to shift included text.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum ExplicitShift {
    /// Don't shift.
    None,
    /// Shift left by removing the given number of leading whitespace chars.
    ///
    /// Does not remove leading non-whitespace chars, i.e. lines with fewer leading whitespace
    /// chars get a smaller shift.
    Left(usize),
    /// Shift right by the given amount, inserting spaces on the left.
    Right(usize),
}

fn common_leading_ws(lines: &[String]) -> String {
    let mut common_ws: Option<String> = None;
    for line in lines {
        if line.is_empty() {
            // Don't include empty lines in the calculation.
            continue;
        }
        let ws = line.chars().take_while(|c| c.is_whitespace());
        if let Some(common) = common_ws {
            common_ws = Some(
                common
                    .chars()
                    .zip(ws)
                    .take_while(|(a, b)| a == b)
                    .map(|(a, _b)| a)
                    .collect(),
            );
        } else {
            common_ws = Some(ws.collect())
        }
    }
    common_ws.unwrap_or_else(String::new)
}

fn calculate_shift(lines: &[String], shift: Shift) -> ExplicitShift {
    match shift {
        Shift::None => ExplicitShift::None,
        Shift::Left(l) => ExplicitShift::Left(l),
        Shift::Right(r) => ExplicitShift::Right(r),
        Shift::Auto => ExplicitShift::Left(common_leading_ws(lines).len()),
    }
}

fn shift_line(l: &str, shift: ExplicitShift) -> Cow<'_, str> {
    match shift {
        ExplicitShift::None => Cow::Borrowed(l),
        ExplicitShift::Right(shift) => {
            let indent = " ".repeat(shift);
            Cow::Owned(format!("{indent}{l}"))
        }
        ExplicitShift::Left(skip) => {
            let mut count = 0;
            let rest = l
                .chars()
                .skip_while(|c| {
                    count += 1;
                    c.is_whitespace() && count <= skip
                })
                .collect::<String>();
            Cow::Owned(rest)
        }
    }
}

fn shift_lines(lines: &[String], shift: Shift) -> Vec<Cow<'_, str>> {
    let shift = calculate_shift(lines, shift);
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
        common_leading_ws, shift_line, take_anchored_lines, take_anchored_lines_with_shift,
        take_lines, take_lines_with_shift, take_rustdoc_include_anchored_lines,
        take_rustdoc_include_lines, ExplicitShift, Shift,
    };

    #[test]
    fn common_leading_ws_test() {
        let tests = [
            (["  line1", "    line2", "  line3"], "  "),
            (["  line1", "    line2", "line3"], ""),
            (["\t\tline1", "\t\t  line2", "\t\tline3"], "\t\t"),
            (["\t line1", " \tline2", "  \t\tline3"], ""),
        ];
        for (lines, want) in tests {
            let lines = lines.into_iter().map(|l| l.to_string()).collect::<Vec<_>>();
            let got = common_leading_ws(&lines);
            assert_eq!(got, want, "for input {lines:?}");
        }
    }

    #[test]
    fn shift_line_test() {
        let s = "    Line with 4 space intro";
        assert_eq!(shift_line(s, ExplicitShift::None), s);
        assert_eq!(
            shift_line(s, ExplicitShift::Left(4)),
            "Line with 4 space intro"
        );
        assert_eq!(
            shift_line(s, ExplicitShift::Left(2)),
            "  Line with 4 space intro"
        );
        assert_eq!(
            shift_line(s, ExplicitShift::Left(6)),
            "Line with 4 space intro"
        );
        assert_eq!(
            shift_line(s, ExplicitShift::Right(2)),
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
        assert_eq!(
            take_lines_with_shift(s, 1..3, Shift::Auto),
            "ipsum\n  dolor"
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
            take_lines_with_shift(s, ..3, Shift::Auto),
            "Lorem\nipsum\n  dolor"
        );
        assert_eq!(
            take_lines_with_shift(s, ..3, Shift::Right(4)),
            "      Lorem\n      ipsum\n        dolor"
        );
        assert_eq!(
            take_lines_with_shift(s, ..3, Shift::Left(4)),
            "Lorem\nipsum\ndolor"
        );
        assert_eq!(take_lines_with_shift(s, .., Shift::None), s);
        assert_eq!(
            take_lines_with_shift(s, .., Shift::Auto),
            "Lorem\nipsum\n  dolor\nsit\namet"
        );
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
            take_anchored_lines_with_shift(s, "test", Shift::Auto),
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
            take_anchored_lines_with_shift(s, "test", Shift::Auto),
            "dolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(4)),
            "dolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(44)),
            "dolor\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::None),
            ""
        );

        let s = "  Lorem\n  ANCHOR: test\n  ipsum\n  ANCHOR: test\n  dolor\n\n\n  sit\n  amet\n  ANCHOR_END: test\n  lorem\n  ipsum";
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::None),
            "  ipsum\n  dolor\n\n\n  sit\n  amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Right(2)),
            "    ipsum\n    dolor\n  \n  \n    sit\n    amet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Left(2)),
            "ipsum\ndolor\n\n\nsit\namet"
        );
        assert_eq!(
            take_anchored_lines_with_shift(s, "test", Shift::Auto),
            "ipsum\ndolor\n\n\nsit\namet"
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
        assert_eq!(
            take_anchored_lines_with_shift(s, "something", Shift::Auto),
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
            "ípsum\ndôlor\nsit\namet\nlorem"
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
