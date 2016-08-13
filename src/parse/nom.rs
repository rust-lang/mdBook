use nom::{self, IResult};

use std::str;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct Link {
    title: String,
    destination: String,
}

/// Parser for markdown links:  `[title](destination)`
fn link(i: &[u8]) -> IResult<&[u8], Link> {
    unimplemented!();
}

/// Parser for parsing the title part of the link: [title](destination)
///                                                ^^^^^^^
///
/// From the Common Mark spec (http://spec.commonmark.org/0.26/#links):
///
/// Brackets are allowed in the link text only if
///   (a) they are backslash-escaped or
///   (b) they appear as a matched pair of brackets,
///       with an open bracket [, a sequence of zero or more inlines, and a close bracket ].
///
fn link_text(i: &[u8]) -> IResult<&[u8], String> {
    map_res!(i,
        map_res!(
            delimited!(
                // Begin with '['
                char!('['),
                // Followed by anything that is not '[' or ']'
                // Make sure to allow escaped brackets and balanced brackets
                recognize!(many1!(alt!(not_unescaped_bracket | balanced_brackets))),
                // End with ']'
                char!(']')
            ),
            str::from_utf8
        ),
        FromStr::from_str
    )
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn balanced_brackets(i: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize!(i,
        delimited!(
            // Begin with '['
            char!('['),
            // Followed by anything that is not '[' or ']'
            // Make sure to allow escaped brackets and balanced brackets
            many0!(alt!(not_unescaped_bracket | balanced_brackets)),
            // End with ']'
            char!(']')
        )
    )
}

fn not_unescaped_bracket(i: &[u8]) -> IResult<&[u8], &[u8]> {
    escaped!(i, is_not!("\\[]"), '\\', is_a_bytes!(&b"[]"[..]))
}

/// Parser for parsing the destination part of the link: [title](destination)
///                                                             ^^^^^^^^^^^^^
fn link_destination(i: &[u8]) -> IResult<&[u8], String> {
    unimplemented!();
}

#[cfg(test)]
mod tests {

    use nom::{self, IResult};
    use nom::Err::{NodePosition, Position};
    use nom::ErrorKind::Escaped;

    // Tests not_unescaped_bracket
    #[test]
    fn not_unescaped_bracket() {
        assert_eq!(super::not_unescaped_bracket(b"a"), IResult::Done(&b""[..], &b"a"[..]));
        assert_eq!(super::not_unescaped_bracket(b"-"), IResult::Done(&b""[..], &b"-"[..]));
        assert_eq!(super::not_unescaped_bracket(b"\\["), IResult::Done(&b""[..], &b"\\["[..]));
        assert_eq!(super::not_unescaped_bracket(b"]"), IResult::Error(NodePosition(Escaped, &[93][..], Box::new(Position(Escaped, &[93][..])))));
    }

    // Tests for balanced brackets
    #[test]
    fn balanced_brackets() {
        assert_eq!(super::balanced_brackets(b"[a]"), nom::IResult::Done(&b""[..], &b"[a]"[..]));
    }

    #[test]
    fn balanced_brackets_empty() {
        assert_eq!(super::balanced_brackets(b"[]"), nom::IResult::Done(&b""[..], &b"[]"[..]));
    }

    #[test]
    fn balanced_brackets_nested() {
        assert_eq!(super::balanced_brackets(b"[abc[a]]"), nom::IResult::Done(&b""[..], &b"[abc[a]]"[..]));
    }

    // Tests for link_text

    #[test]
    fn link_text_one_ch() {
        assert_eq!(super::link_text(b"[a]"), nom::IResult::Done(&b""[..], String::from("a")));
    }

    #[test]
    fn link_text_multi_ch() {
        assert_eq!(super::link_text(b"[Intro]"), nom::IResult::Done(&b""[..], String::from("Intro")));
    }

    #[test]
    fn link_text_sp_ch() {
        assert_eq!(super::link_text(b"[Intro!]"), nom::IResult::Done(&b""[..], String::from("Intro!")));
    }

    #[test]
    fn link_text_unicode() {
        assert_eq!(super::link_text("[Heizölrückstoßabdämpfung]".as_bytes()),
                   nom::IResult::Done(&b""[..], String::from("Heizölrückstoßabdämpfung")));
        assert_eq!(super::link_text("[Здравствуйте]".as_bytes()),
                   nom::IResult::Done(&b""[..], String::from("Здравствуйте")));
    }

    #[test]
    fn link_text_brackets() {
        assert_eq!(super::link_text(b"[Intro[]]"), nom::IResult::Done(&b""[..], String::from("Intro[]")));
        assert_eq!(super::link_text(br"[Intro\]]"), nom::IResult::Done(&b""[..], String::from("Intro[")));
    }
}
