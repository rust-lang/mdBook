use crate::errors::*;
use crate::utils::fs::file_to_string;
use crate::utils::take_lines;
use regex::{CaptureMatches, Captures, Regex};
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};
use std::path::{Path, PathBuf};

use super::{Preprocessor, PreprocessorContext};
use crate::book::{Book, BookItem};

const ESCAPE_CHAR: char = '\\';
const MAX_LINK_NESTED_DEPTH: usize = 10;

/// A preprocessor for expanding the `{{# playpen}}` and `{{# include}}`
/// helpers in a chapter.
#[derive(Default)]
pub struct LinkPreprocessor;

impl LinkPreprocessor {
    pub(crate) const NAME: &'static str = "links";

    /// Create a new `LinkPreprocessor`.
    pub fn new() -> Self {
        LinkPreprocessor
    }
}

impl Preprocessor for LinkPreprocessor {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let src_dir = ctx.root.join(&ctx.config.book.src);

        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut ch) = *section {
                let base = ch
                    .path
                    .parent()
                    .map(|dir| src_dir.join(dir))
                    .expect("All book items have a parent");

                let content = replace_all(&ch.content, base, &ch.path, 0);
                ch.content = content;
            }
        });

        Ok(book)
    }
}

fn replace_all<P1, P2>(s: &str, path: P1, source: P2, depth: usize) -> String
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // When replacing one thing in a string by something with a different length,
    // the indices after that will not correspond,
    // we therefore have to store the difference to correct this
    let path = path.as_ref();
    let source = source.as_ref();
    let mut previous_end_index = 0;
    let mut replaced = String::new();

    for playpen in find_links(s) {
        replaced.push_str(&s[previous_end_index..playpen.start_index]);

        match playpen.render_with_path(&path) {
            Ok(new_content) => {
                if depth < MAX_LINK_NESTED_DEPTH {
                    if let Some(rel_path) = playpen.link.relative_path(path) {
                        replaced.push_str(&replace_all(&new_content, rel_path, source, depth + 1));
                    } else {
                        replaced.push_str(&new_content);
                    }
                } else {
                    error!(
                        "Stack depth exceeded in {}. Check for cyclic includes",
                        source.display()
                    );
                }
                previous_end_index = playpen.end_index;
            }
            Err(e) => {
                error!("Error updating \"{}\", {}", playpen.link_text, e);
                for cause in e.iter().skip(1) {
                    warn!("Caused By: {}", cause);
                }

                // This should make sure we include the raw `{{# ... }}` snippet
                // in the page content if there are any errors.
                previous_end_index = playpen.start_index;
            }
        }
    }

    replaced.push_str(&s[previous_end_index..]);
    replaced
}

#[derive(PartialEq, Debug, Clone)]
enum LinkType<'a> {
    Escaped,
    IncludeRange(PathBuf, Range<usize>),
    IncludeRangeFrom(PathBuf, RangeFrom<usize>),
    IncludeRangeTo(PathBuf, RangeTo<usize>),
    IncludeRangeFull(PathBuf, RangeFull),
    Playpen(PathBuf, Vec<&'a str>),
}

impl<'a> LinkType<'a> {
    fn relative_path<P: AsRef<Path>>(self, base: P) -> Option<PathBuf> {
        let base = base.as_ref();
        match self {
            LinkType::Escaped => None,
            LinkType::IncludeRange(p, _) => Some(return_relative_path(base, &p)),
            LinkType::IncludeRangeFrom(p, _) => Some(return_relative_path(base, &p)),
            LinkType::IncludeRangeTo(p, _) => Some(return_relative_path(base, &p)),
            LinkType::IncludeRangeFull(p, _) => Some(return_relative_path(base, &p)),
            LinkType::Playpen(p, _) => Some(return_relative_path(base, &p)),
        }
    }
}
fn return_relative_path<P: AsRef<Path>>(base: P, relative: P) -> PathBuf {
    base.as_ref()
        .join(relative)
        .parent()
        .expect("Included file should not be /")
        .to_path_buf()
}

fn parse_include_path(path: &str) -> LinkType<'static> {
    let mut parts = path.split(':');
    let path = parts.next().unwrap().into();
    // subtract 1 since line numbers usually begin with 1
    let start = parts
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|val| val.saturating_sub(1));
    let end = parts.next();
    let has_end = end.is_some();
    let end = end.and_then(|s| s.parse::<usize>().ok());
    match start {
        Some(start) => match end {
            Some(end) => LinkType::IncludeRange(path, Range { start, end }),
            None => {
                if has_end {
                    LinkType::IncludeRangeFrom(path, RangeFrom { start })
                } else {
                    LinkType::IncludeRange(
                        path,
                        Range {
                            start,
                            end: start + 1,
                        },
                    )
                }
            }
        },
        None => match end {
            Some(end) => LinkType::IncludeRangeTo(path, RangeTo { end }),
            None => LinkType::IncludeRangeFull(path, RangeFull),
        },
    }
}

#[derive(PartialEq, Debug, Clone)]
struct Link<'a> {
    start_index: usize,
    end_index: usize,
    link: LinkType<'a>,
    link_text: &'a str,
}

impl<'a> Link<'a> {
    fn from_capture(cap: Captures<'a>) -> Option<Link<'a>> {
        let link_type = match (cap.get(0), cap.get(1), cap.get(2)) {
            (_, Some(typ), Some(rest)) => {
                let mut path_props = rest.as_str().split_whitespace();
                let file_arg = path_props.next();
                let props: Vec<&str> = path_props.collect();

                match (typ.as_str(), file_arg) {
                    ("include", Some(pth)) => Some(parse_include_path(pth)),
                    ("playpen", Some(pth)) => Some(LinkType::Playpen(pth.into(), props)),
                    _ => None,
                }
            }
            (Some(mat), None, None) if mat.as_str().starts_with(ESCAPE_CHAR) => {
                Some(LinkType::Escaped)
            }
            _ => None,
        };

        link_type.and_then(|lnk| {
            cap.get(0).map(|mat| Link {
                start_index: mat.start(),
                end_index: mat.end(),
                link: lnk,
                link_text: mat.as_str(),
            })
        })
    }

    fn render_with_path<P: AsRef<Path>>(&self, base: P) -> Result<String> {
        let base = base.as_ref();
        match self.link {
            // omit the escape char
            LinkType::Escaped => Ok((&self.link_text[1..]).to_owned()),
            LinkType::IncludeRange(ref pat, ref range) => {
                let target = base.join(pat);

                file_to_string(&target)
                    .map(|s| take_lines(&s, range.clone()))
                    .chain_err(|| {
                        format!(
                            "Could not read file for link {} ({})",
                            self.link_text,
                            target.display(),
                        )
                    })
            }
            LinkType::IncludeRangeFrom(ref pat, ref range) => {
                let target = base.join(pat);

                file_to_string(&target)
                    .map(|s| take_lines(&s, range.clone()))
                    .chain_err(|| {
                        format!(
                            "Could not read file for link {} ({})",
                            self.link_text,
                            target.display(),
                        )
                    })
            }
            LinkType::IncludeRangeTo(ref pat, ref range) => {
                let target = base.join(pat);

                file_to_string(&target)
                    .map(|s| take_lines(&s, *range))
                    .chain_err(|| {
                        format!(
                            "Could not read file for link {} ({})",
                            self.link_text,
                            target.display(),
                        )
                    })
            }
            LinkType::IncludeRangeFull(ref pat, _) => {
                let target = base.join(pat);

                file_to_string(&target).chain_err(|| {
                    format!(
                        "Could not read file for link {} ({})",
                        self.link_text,
                        target.display()
                    )
                })
            }
            LinkType::Playpen(ref pat, ref attrs) => {
                let target = base.join(pat);

                let contents = file_to_string(&target).chain_err(|| {
                    format!(
                        "Could not read file for link {} ({})",
                        self.link_text,
                        target.display()
                    )
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

fn find_links(contents: &str) -> LinkIter<'_> {
    // lazily compute following regex
    // r"\\\{\{#.*\}\}|\{\{#([a-zA-Z0-9]+)\s*([a-zA-Z0-9_.\-:/\\\s]+)\}\}")?;
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)                     # insignificant whitespace mode
            \\\{\{\#.*\}\}             # match escaped link
            |                          # or
            \{\{\s*                    # link opening parens and whitespace
            \#([a-zA-Z0-9]+)           # link type
            \s+                        # separating whitespace
            ([a-zA-Z0-9\s_.\-:/\\]+)   # link target path and space separated properties
            \s*\}\}                    # whitespace and link closing parens"
        )
        .unwrap();
    }
    LinkIter(RE.captures_iter(contents))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_all_escaped() {
        let start = r"
        Some text over here.
        ```hbs
        \{{#include file.rs}} << an escaped link!
        ```";
        let end = r"
        Some text over here.
        ```hbs
        {{#include file.rs}} << an escaped link!
        ```";
        assert_eq!(replace_all(start, "", "", 0), end);
    }

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

        assert_eq!(
            res,
            vec![
                Link {
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
                },
            ]
        );
    }

    #[test]
    fn test_find_links_with_range() {
        let s = "Some random text with {{#include file.rs:10:20}}...";
        let res = find_links(s).collect::<Vec<_>>();
        println!("\nOUTPUT: {:?}\n", res);
        assert_eq!(
            res,
            vec![Link {
                start_index: 22,
                end_index: 48,
                link: LinkType::IncludeRange(PathBuf::from("file.rs"), 9..20),
                link_text: "{{#include file.rs:10:20}}",
            }]
        );
    }

    #[test]
    fn test_find_links_with_line_number() {
        let s = "Some random text with {{#include file.rs:10}}...";
        let res = find_links(s).collect::<Vec<_>>();
        println!("\nOUTPUT: {:?}\n", res);
        assert_eq!(
            res,
            vec![Link {
                start_index: 22,
                end_index: 45,
                link: LinkType::IncludeRange(PathBuf::from("file.rs"), 9..10),
                link_text: "{{#include file.rs:10}}",
            }]
        );
    }

    #[test]
    fn test_find_links_with_from_range() {
        let s = "Some random text with {{#include file.rs:10:}}...";
        let res = find_links(s).collect::<Vec<_>>();
        println!("\nOUTPUT: {:?}\n", res);
        assert_eq!(
            res,
            vec![Link {
                start_index: 22,
                end_index: 46,
                link: LinkType::IncludeRangeFrom(PathBuf::from("file.rs"), 9..),
                link_text: "{{#include file.rs:10:}}",
            }]
        );
    }

    #[test]
    fn test_find_links_with_to_range() {
        let s = "Some random text with {{#include file.rs::20}}...";
        let res = find_links(s).collect::<Vec<_>>();
        println!("\nOUTPUT: {:?}\n", res);
        assert_eq!(
            res,
            vec![Link {
                start_index: 22,
                end_index: 46,
                link: LinkType::IncludeRangeTo(PathBuf::from("file.rs"), ..20),
                link_text: "{{#include file.rs::20}}",
            }]
        );
    }

    #[test]
    fn test_find_links_with_full_range() {
        let s = "Some random text with {{#include file.rs::}}...";
        let res = find_links(s).collect::<Vec<_>>();
        println!("\nOUTPUT: {:?}\n", res);
        assert_eq!(
            res,
            vec![Link {
                start_index: 22,
                end_index: 44,
                link: LinkType::IncludeRangeFull(PathBuf::from("file.rs"), ..),
                link_text: "{{#include file.rs::}}",
            }]
        );
    }

    #[test]
    fn test_find_links_with_no_range_specified() {
        let s = "Some random text with {{#include file.rs}}...";
        let res = find_links(s).collect::<Vec<_>>();
        println!("\nOUTPUT: {:?}\n", res);
        assert_eq!(
            res,
            vec![Link {
                start_index: 22,
                end_index: 42,
                link: LinkType::IncludeRangeFull(PathBuf::from("file.rs"), ..),
                link_text: "{{#include file.rs}}",
            }]
        );
    }

    #[test]
    fn test_find_links_escaped_link() {
        let s = "Some random text with escaped playpen \\{{#playpen file.rs editable}} ...";

        let res = find_links(s).collect::<Vec<_>>();
        println!("\nOUTPUT: {:?}\n", res);

        assert_eq!(
            res,
            vec![Link {
                start_index: 38,
                end_index: 68,
                link: LinkType::Escaped,
                link_text: "\\{{#playpen file.rs editable}}",
            }]
        );
    }

    #[test]
    fn test_find_playpens_with_properties() {
        let s = "Some random text with escaped playpen {{#playpen file.rs editable }} and some \
                 more\n text {{#playpen my.rs editable no_run should_panic}} ...";

        let res = find_links(s).collect::<Vec<_>>();
        println!("\nOUTPUT: {:?}\n", res);
        assert_eq!(
            res,
            vec![
                Link {
                    start_index: 38,
                    end_index: 68,
                    link: LinkType::Playpen(PathBuf::from("file.rs"), vec!["editable"]),
                    link_text: "{{#playpen file.rs editable }}",
                },
                Link {
                    start_index: 89,
                    end_index: 136,
                    link: LinkType::Playpen(
                        PathBuf::from("my.rs"),
                        vec!["editable", "no_run", "should_panic"],
                    ),
                    link_text: "{{#playpen my.rs editable no_run should_panic}}",
                },
            ]
        );
    }

    #[test]
    fn test_find_all_link_types() {
        let s = "Some random text with escaped playpen {{#include file.rs}} and \\{{#contents are \
                 insignifficant in escaped link}} some more\n text  {{#playpen my.rs editable \
                 no_run should_panic}} ...";

        let res = find_links(s).collect::<Vec<_>>();
        println!("\nOUTPUT: {:?}\n", res);
        assert_eq!(res.len(), 3);
        assert_eq!(
            res[0],
            Link {
                start_index: 38,
                end_index: 58,
                link: LinkType::IncludeRangeFull(PathBuf::from("file.rs"), ..),
                link_text: "{{#include file.rs}}",
            }
        );
        assert_eq!(
            res[1],
            Link {
                start_index: 63,
                end_index: 112,
                link: LinkType::Escaped,
                link_text: "\\{{#contents are insignifficant in escaped link}}",
            }
        );
        assert_eq!(
            res[2],
            Link {
                start_index: 130,
                end_index: 177,
                link: LinkType::Playpen(
                    PathBuf::from("my.rs"),
                    vec!["editable", "no_run", "should_panic"]
                ),
                link_text: "{{#playpen my.rs editable no_run should_panic}}",
            }
        );
    }

}
