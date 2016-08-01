use nom::{self, IResult, alphanumeric};

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
///
/// From the Common Mark spec (http://spec.commonmark.org/0.26/#links):
///
/// Brackets are allowed in the link text only if
///   (a) they are backslash-escaped or
///   (b) they appear as a matched pair of brackets,
///       with an open bracket [, a sequence of zero or more inlines, and a close bracket ].
///                                        ^^^^^^^
fn link_text(i: &[u8]) -> IResult<&[u8], String> {
    map_res!(i, map_res!(delimited!(char!('['), is_not!("[]"), char!(']')), str::from_utf8), FromStr::from_str)
}

/// Parser for parsing the destination part of the link: [title](destination)
///                                                             ^^^^^^^^^^^^^
fn link_destination(i: &[u8]) -> IResult<&[u8], String> {
    unimplemented!();
}

#[cfg(test)]
mod tests {

    use nom::{self, IResult};

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
        assert_eq!(super::link_text(br"[Intro\[]"), nom::IResult::Done(&b""[..], String::from("Intro[")));
    }
}
