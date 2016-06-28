use nom::*;

use std::str;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    title: String,
    destination: String,

    children: Option<Vec<Link>>,
}

pub fn summary(i: &[u8]) -> IResult<&[u8], (Vec<Link>, Vec<Link>, Vec<Link>)> {
    chain!(i,
        header? ~
        r: tuple!(preface, chapters, preface),
        || r
    )
}

fn header(i: &[u8]) -> IResult<&[u8], ()> {
    chain!(i,
        tag!("#") ~
        space ~
        many1!(not_line_ending) ~
        line_ending,
        || {}
    )
}

fn chapters(i: &[u8]) -> IResult<&[u8], Vec<Link>> {
    unimplemented!()
}

fn preface(i: &[u8]) -> IResult<&[u8], Vec<Link>> {
    many0!(i,
        chain!(
            link: link ~
            many1!(alt!(multispace | eof | line_ending)),
            || {
                println!("{:?}", link);
                link
            }
        )
    )
}

fn list_link(i: &[u8]) -> IResult<&[u8], Link> {
    chain!(i,
        tag!("- ") ~
        link: link,
        || { link }
    )
}

/// Parser for markdown links:  `[title](destination)`
fn link(i: &[u8]) -> IResult<&[u8], Link> {
    chain!(i,
        tag!("[") ~
        title: map_res!(take_until!("]("), str::from_utf8) ~
        tag!("](") ~
        destination: map_res!(take_until!(")"), str::from_utf8) ~
        tag!(")"),
        || {
            Link {
                title: title.to_string(),
                destination: destination.to_string(),
                children: None,
            }
        }
    )
}



#[cfg(test)]
mod tests {

    use super::Link;
    use nom::{self, IResult};
    use nom::Err::{NodePosition, Position};
    use nom::ErrorKind::Escaped;

    // Test preface
    #[test]
    fn preface() {

        let preface = r#"[a](a.md)
[b](b.md)


[c](c.md)

"#;
        assert_eq!(
            super::preface(preface.as_bytes()).unwrap().1,
            vec![
                Link { title: String::from("a"), destination: String::from("a.md"), children: None },
                Link { title: String::from("b"), destination: String::from("b.md"), children: None },
                Link { title: String::from("c"), destination: String::from("c.md"), children: None }
            ]
        );
    }

    // Tests link

    #[test]
    fn link() {
        assert_eq!(
            super::link(b"[a](a.md)").unwrap().1,
            Link { title: String::from("a"), destination: String::from("a.md"), children: None  }
        );
    }

    #[test]
    fn link_sp_ch() {
        assert_eq!(
            super::link(b"[Intro!](path-1.md)").unwrap().1,
            Link { title: String::from("Intro!"), destination: String::from("path-1.md"), children: None  }
        );
    }

    #[test]
    fn link_unicode() {
        assert_eq!(
            super::link("[Heizölrückstoßabdämpfung](path-1.md)".as_bytes()).unwrap().1,
            Link { title: String::from("Heizölrückstoßabdämpfung"), destination: String::from("path-1.md"), children: None  }
        );

        assert_eq!(
            super::link("[Здравствуйте](path-1.md)".as_bytes()).unwrap().1,
            Link { title: String::from("Здравствуйте"), destination: String::from("path-1.md"), children: None  }
        );
    }

    #[test]
    fn link_brackets() {
        assert_eq!(
            super::link(b"[Intro[]](path-1.md)").unwrap().1,
            Link { title: String::from("Intro[]"), destination: String::from("path-1.md"), children: None  }
        );

        // TODO: modify the parser to pass the following test...
        // assert_eq!(
        //     super::link(br"[Intro\]](path-1.md)").unwrap().1,
        //     Link { title: String::from("Intro["), destination: String::from("path-1.md"), children: None  }
        // );
    }
}
