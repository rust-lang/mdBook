extern crate nom;

use self::nom::IResult;

#[derive(Debug, Clone)]
struct Link {
    title: String,
    destination: String,
}

/// Parser for markdown links:  `[title](destination)`
/// From the Common Mark spec:
///
/// Brackets are allowed in the link text only if
///   (a) they are backslash-escaped or
///   (b) they appear as a matched pair of brackets,
///       with an open bracket [, a sequence of zero or more inlines, and a close bracket ].
///
fn link(i: &[u8]) -> IResult<&[u8], Link> {
    unimplemented!();
}

/// Parser for parsing the title part of the link: [title](destination)
///                                                ^^^^^^^
fn link_text(i: &[u8]) -> IResult<&[u8], String> {
    unimplemented!();
}

fn link_destination(i: &[u8]) -> IResult<&[u8], String> {
    unimplemented!();
}
