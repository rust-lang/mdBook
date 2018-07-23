use itertools::Itertools;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

// This trait is already contained in the standard lib, however it is unstable.
// TODO: Remove when the `collections_range` feature stabilises
// (https://github.com/rust-lang/rust/issues/30877)
pub trait RangeArgument<T: ?Sized> {
    fn start(&self) -> Option<&T>;
    fn end(&self) -> Option<&T>;
}

impl<T: ?Sized> RangeArgument<T> for RangeFull {
    fn start(&self) -> Option<&T> {
        None
    }
    fn end(&self) -> Option<&T> {
        None
    }
}

impl<T> RangeArgument<T> for RangeFrom<T> {
    fn start(&self) -> Option<&T> {
        Some(&self.start)
    }
    fn end(&self) -> Option<&T> {
        None
    }
}

impl<T> RangeArgument<T> for RangeTo<T> {
    fn start(&self) -> Option<&T> {
        None
    }
    fn end(&self) -> Option<&T> {
        Some(&self.end)
    }
}

impl<T> RangeArgument<T> for Range<T> {
    fn start(&self) -> Option<&T> {
        Some(&self.start)
    }
    fn end(&self) -> Option<&T> {
        Some(&self.end)
    }
}

/// Take a range of lines from a string.
pub fn take_lines<R: RangeArgument<usize>>(s: &str, range: R) -> String {
    let start = *range.start().unwrap_or(&0);
    let mut lines = s.lines().skip(start);
    match range.end() {
        Some(&end) => lines.take(end.saturating_sub(start)).join("\n"),
        None => lines.join("\n"),
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
