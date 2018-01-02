use std::path::{Path, PathBuf};
use regex::{CaptureMatches, Captures, Regex};
use utils::fs::file_to_string;
use errors::*;

const ESCAPE_CHAR: char = '\\';

pub fn replace_all<P: AsRef<Path>>(s: &str, path: P) -> Result<String> {
    // When replacing one thing in a string by something with a different length,
    // the indices after that will not correspond,
    // we therefore have to store the difference to correct this
    let mut previous_end_index = 0;
    let mut replaced = String::new();

    for playpen in find_links(s) {
        replaced.push_str(&s[previous_end_index..playpen.start_index]);
        replaced.push_str(&playpen.render_with_path(&path)?);
        previous_end_index = playpen.end_index;
    }

    replaced.push_str(&s[previous_end_index..]);
    Ok(replaced)
}

#[derive(PartialOrd, PartialEq, Debug, Clone)]
enum LinkType<'a> {
    Escaped,
    IncludeRangeFrom(PathBuf, usize),
    IncludeRange(PathBuf, usize, usize),
    Playpen(PathBuf, Vec<&'a str>),
}

#[derive(PartialOrd, PartialEq, Debug, Clone)]
struct Link<'a> {
    start_index: usize,
    end_index: usize,
    link: LinkType<'a>,
    link_text: &'a str,
}

impl<'a> Link<'a> {
    fn parse_include_path(path: PathBuf) -> Option<LinkType<'a>> {
        path.to_str().map(|p| {
            let mut parts = p.split(':');
            let p = parts.next().unwrap();
            let from = parts.next().and_then(|s| s.parse::<usize>().ok());
            let to = parts.next().and_then(|s| s.parse::<usize>().ok());
            match from {
                Some(from) => {
                    match to {
                        Some(to) => LinkType::IncludeRange(p.into(), from, to),
                        None => LinkType::IncludeRangeFrom(p.into(), from),
                    }
                }
                None => {
                    match to {
                        Some(to) => LinkType::IncludeRange(p.into(), 0, to),
                        None => LinkType::IncludeRangeFrom(p.into(), 0),
                    }
                }
            }
        })
    }

    fn from_capture(cap: Captures<'a>) -> Option<Link<'a>> {
        let link_type = match (cap.get(0), cap.get(1), cap.get(2)) {
            (_, Some(typ), Some(rest)) => {
                let mut path_props = rest.as_str().split_whitespace();
                let file_path = path_props.next().map(PathBuf::from);
                let props: Vec<&str> = path_props.collect();

                match (typ.as_str(), file_path) {
                    ("include", Some(pth)) => Self::parse_include_path(pth),
                    ("playpen", Some(pth)) => Some(LinkType::Playpen(pth, props)),
                    _ => None,
                }
            }
            (Some(mat), None, None) if mat.as_str().starts_with(ESCAPE_CHAR) => Some(
                LinkType::Escaped,
            ),
            _ => None,
        };

        link_type.and_then(|lnk| {
            cap.get(0).map(|mat| {
                Link {
                    start_index: mat.start(),
                    end_index: mat.end(),
                    link: lnk,
                    link_text: mat.as_str(),
                }
            })
        })
    }

    fn join_lines<'b, I: IntoIterator<Item = &'b str>>(lines: I) -> String {
        let mut lines = lines.into_iter();
        let first = String::from(lines.next().unwrap_or(""));
        lines.fold(first, |acc, s| acc + "\n" + s)
    }

    fn take_lines(s: String, from: usize, to_inclusive: Option<usize>) -> String {
        let lines = s.split('\n').skip(from);
        match to_inclusive {
            Some(to) => Self::join_lines(lines.take(to.checked_sub(from).unwrap_or(0))),
            None => Self::join_lines(lines),
        }
    }

    fn render_with_path<P: AsRef<Path>>(&self, base: P) -> Result<String> {
        let base = base.as_ref();
        match self.link {
            // omit the escape char
            LinkType::Escaped => Ok((&self.link_text[1..]).to_owned()),
            LinkType::IncludeRangeFrom(ref pat, from) => {
                file_to_string(base.join(pat))
                    .map(|s| Self::take_lines(s, from, None))
                    .chain_err(|| {
                        format!("Could not read file for link {}", self.link_text)
                    })
            }
            LinkType::IncludeRange(ref pat, from, to) => {
                file_to_string(base.join(pat))
                    .map(|s| Self::take_lines(s, from, Some(to)))
                    .chain_err(|| {
                        format!("Could not read file for link {}", self.link_text)
                    })
            }
            LinkType::Playpen(ref pat, ref attrs) => {
                let contents = file_to_string(base.join(pat)).chain_err(|| {
                    format!("Could not read file for link {}", self.link_text)
                })?;
                let ftype = if !attrs.is_empty() { "rust," } else { "rust" };
                Ok(format!(
                    "```{}{}\n{}\n```\n",
                    ftype,
                    attrs.join(","),
                    contents
                ))
            }
        }
    }
}

struct LinkIter<'a>(CaptureMatches<'a, 'a>);

impl<'a> Iterator for LinkIter<'a> {
    type Item = Link<'a>;
    fn next(&mut self) -> Option<Link<'a>> {
        for cap in &mut self.0 {
            if let Some(inc) = Link::from_capture(cap) {
                return Some(inc);
            }
        }
        None
    }
}

fn find_links(contents: &str) -> LinkIter {
    // lazily compute following regex
    // r"\\\{\{#.*\}\}|\{\{#([a-zA-Z0-9]+)\s*([a-zA-Z0-9_.\-:/\\\s]+)\}\}")?;
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?x) # insignificant whitespace mode
                    \\\{\{\#.*\}\}               # match escaped link
                    |                            # or
                    \{\{\s*                      # link opening parens and whitespace
                      \#([a-zA-Z0-9]+)           # link type
                      \s+                        # separating whitespace
                      ([a-zA-Z0-9\s_.\-:/\\]+)   # link target path and space separated properties
                    \s*\}\}                      # whitespace and link closing parens
                                 ").unwrap();
    }
    LinkIter(RE.captures_iter(contents))
}

// ---------------------------------------------------------------------------------
//      Tests
//

#[test]
fn test_find_links_no_link() {
    let s = "Some random text without link...";
    assert!(find_links(s).collect::<Vec<_>>() == vec![]);
}

#[test]
fn test_find_links_partial_link() {
    let s = "Some random text with {{#playpen...";
    assert!(find_links(s).collect::<Vec<_>>() == vec![]);
    let s = "Some random text with {{#include...";
    assert!(find_links(s).collect::<Vec<_>>() == vec![]);
    let s = "Some random text with \\{{#include...";
    assert!(find_links(s).collect::<Vec<_>>() == vec![]);
}

#[test]
fn test_find_links_empty_link() {
    let s = "Some random text with {{#playpen}} and {{#playpen   }} {{}} {{#}}...";
    assert!(find_links(s).collect::<Vec<_>>() == vec![]);
}

#[test]
fn test_find_links_unknown_link_type() {
    let s = "Some random text with {{#playpenz ar.rs}} and {{#incn}} {{baz}} {{#bar}}...";
    assert!(find_links(s).collect::<Vec<_>>() == vec![]);
}

#[test]
fn test_find_links_simple_link() {
    let s = "Some random text with {{#playpen file.rs}} and {{#playpen test.rs }}...";

    let res = find_links(s).collect::<Vec<_>>();
    println!("\nOUTPUT: {:?}\n", res);

    assert_eq!(res,
               vec![Link {
                        start_index: 22,
                        end_index: 42,
                        link: LinkType::Playpen(PathBuf::from("file.rs"), vec![]),
                        link_text: "{{#playpen file.rs}}",
                    },
                    Link {
                        start_index: 47,
                        end_index: 68,
                        link: LinkType::Playpen(PathBuf::from("test.rs"), vec![]),
                        link_text: "{{#playpen test.rs }}",
                    }]);
}

#[test]
fn test_find_links_with_range() {
    let s = "Some random text with {{#include file.rs:10:20}}...";
    let res = find_links(s).collect::<Vec<_>>();
    println!("\nOUTPUT: {:?}\n", res);
    assert_eq!(
        res,
        vec![
            Link {
                start_index: 22,
                end_index: 48,
                link: LinkType::IncludeRange(PathBuf::from("file.rs"), 10, 20),
                link_text: "{{#include file.rs:10:20}}",
            },
        ]
    );

    let s = "Some random text with {{#include file.rs:10:}}...";
    let res = find_links(s).collect::<Vec<_>>();
    println!("\nOUTPUT: {:?}\n", res);
    assert_eq!(
        res,
        vec![
            Link {
                start_index: 22,
                end_index: 46,
                link: LinkType::IncludeRangeFrom(PathBuf::from("file.rs"), 10),
                link_text: "{{#include file.rs:10:}}",
            },
        ]
    );

    let s = "Some random text with {{#include file.rs::20}}...";
    let res = find_links(s).collect::<Vec<_>>();
    println!("\nOUTPUT: {:?}\n", res);
    assert_eq!(
        res,
        vec![
            Link {
                start_index: 22,
                end_index: 46,
                link: LinkType::IncludeRange(PathBuf::from("file.rs"), 0, 20),
                link_text: "{{#include file.rs::20}}",
            },
        ]
    );

    let s = "Some random text with {{#include file.rs::}}...";
    let res = find_links(s).collect::<Vec<_>>();
    println!("\nOUTPUT: {:?}\n", res);
    assert_eq!(
        res,
        vec![
            Link {
                start_index: 22,
                end_index: 44,
                link: LinkType::IncludeRangeFrom(PathBuf::from("file.rs"), 0),
                link_text: "{{#include file.rs::}}",
            },
        ]
    );
}


#[test]
fn test_find_links_escaped_link() {
    let s = "Some random text with escaped playpen \\{{#playpen file.rs editable}} ...";

    let res = find_links(s).collect::<Vec<_>>();
    println!("\nOUTPUT: {:?}\n", res);

    assert_eq!(res,
               vec![Link {
                        start_index: 38,
                        end_index: 68,
                        link: LinkType::Escaped,
                        link_text: "\\{{#playpen file.rs editable}}",
                    }]);
}

#[test]
fn test_find_playpens_with_properties() {
    let s = "Some random text with escaped playpen {{#playpen file.rs editable }} and some more\n \
             text {{#playpen my.rs editable no_run should_panic}} ...";

    let res = find_links(s).collect::<Vec<_>>();
    println!("\nOUTPUT: {:?}\n", res);
    assert_eq!(res,
               vec![Link {
                        start_index: 38,
                        end_index: 68,
                        link: LinkType::Playpen(PathBuf::from("file.rs"), vec!["editable"]),
                        link_text: "{{#playpen file.rs editable }}",
                    },
                    Link {
                        start_index: 89,
                        end_index: 136,
                        link: LinkType::Playpen(PathBuf::from("my.rs"),
                                                vec!["editable", "no_run", "should_panic"]),
                        link_text: "{{#playpen my.rs editable no_run should_panic}}",
                    }]);
}

#[test]
fn test_find_all_link_types() {
    let s = "Some random text with escaped playpen {{#include file.rs}} and \\{{#contents are \
             insignifficant in escaped link}} some more\n text  {{#playpen my.rs editable no_run \
             should_panic}} ...";

    let res = find_links(s).collect::<Vec<_>>();
    println!("\nOUTPUT: {:?}\n", res);
    assert_eq!(res.len(), 3);
    assert_eq!(res[0],
               Link {
                   start_index: 38,
                   end_index: 58,
                   link: LinkType::IncludeRangeFrom(PathBuf::from("file.rs"), 0),
                   link_text: "{{#include file.rs}}",
               });
    assert_eq!(res[1],
               Link {
                   start_index: 63,
                   end_index: 112,
                   link: LinkType::Escaped,
                   link_text: "\\{{#contents are insignifficant in escaped link}}",
               });
    assert_eq!(res[2],
               Link {
                   start_index: 130,
                   end_index: 177,
                   link: LinkType::Playpen(PathBuf::from("my.rs"),
                                           vec!["editable", "no_run", "should_panic"]),
                   link_text: "{{#playpen my.rs editable no_run should_panic}}",
               });
}
