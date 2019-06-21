use itertools::Itertools;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

/// Take a range of lines from a string.
pub fn take_lines<R: RangeBounds<usize>>(s: &str, range: R) -> String {
    let start = match range.start_bound() {
        Excluded(&n) => n + 1,
        Included(&n) => n,
        Unbounded => 0,
    };
    let mut lines = s.lines().skip(start);
    match range.end_bound() {
        Excluded(end) => lines.take(end.saturating_sub(start)).join("\n"),
        Included(end) => lines.take((end + 1).saturating_sub(start)).join("\n"),
        Unbounded => lines.join("\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::take_lines;

    #[test]
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
}
