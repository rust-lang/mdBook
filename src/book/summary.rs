use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use memchr::{self, Memchr};
use pulldown_cmark::{self, Event, Tag};
use errors::*;

/// Parse the text from a `SUMMARY.md` file into a sort of "recipe" to be
/// used when loading a book from disk.
///
/// # Summary Format
///
/// **Title:** It's common practice to begin with a title, generally
/// "# Summary". It's not mandatory and the parser (currently) ignores it, so
/// you can too if you feel like it.
///
/// **Prefix Chapter:** Before the main numbered chapters you can add a couple
/// of elements that will not be numbered. This is useful for forewords,
/// introductions, etc. There are however some constraints. You can not nest
/// prefix chapters, they should all be on the root level. And you can not add
/// prefix chapters once you have added numbered chapters.
///
/// ```markdown
/// [Title of prefix element](relative/path/to/markdown.md)
/// ```
///
/// **Numbered Chapter:** Numbered chapters are the main content of the book,
/// they
/// will be numbered and can be nested, resulting in a nice hierarchy (chapters,
/// sub-chapters, etc.)
///
/// ```markdown
/// - [Title of the Chapter](relative/path/to/markdown.md)
/// ```
///
/// You can either use - or * to indicate a numbered chapter, the parser doesn't
/// care but you'll probably want to stay consistent.
///
/// **Suffix Chapter:** After the numbered chapters you can add a couple of
/// non-numbered chapters. They are the same as prefix chapters but come after
/// the numbered chapters instead of before.
///
/// All other elements are unsupported and will be ignored at best or result in
/// an error.
pub fn parse_summary(summary: &str) -> Result<Summary> {
    let parser = SummaryParser::new(summary);
    parser.parse()
}

/// The parsed `SUMMARY.md`, specifying how the book should be laid out.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    /// An optional title for the `SUMMARY.md`, currently just ignored.
    pub title: Option<String>,
    /// Chapters before the main text (e.g. an introduction).
    pub prefix_chapters: Vec<SummaryItem>,
    /// The main chapters in the document.
    pub numbered_chapters: Vec<SummaryItem>,
    /// Items which come after the main document (e.g. a conclusion).
    pub suffix_chapters: Vec<SummaryItem>,
}

/// A linked chapter in the `SUMMARY.md`, possibly with nested entries.
///
/// This is roughly the equivalent of `- [Some section](./path/to/file.md)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// The name of the chapter.
    pub name: String,
    /// The location of the chapter's source file, taking the book's `src`
    /// directory as the root.
    pub location: PathBuf,
    /// The section number, if this chapter is in the numbered section.
    pub number: Option<SectionNumber>,
    /// Any nested items this chapter may contain.
    pub nested_items: Vec<SummaryItem>,
}

impl Link {
    /// Create a new link with no nested items.
    pub fn new<S: Into<String>, P: AsRef<Path>>(name: S, location: P) -> Self {
        Self {
            name: name.into(),
            location: location.as_ref().to_path_buf(),
            number: None,
            nested_items: Vec::new(),
        }
    }
}

impl Default for Link {
    fn default() -> Self {
        Self {
            name: String::new(),
            location: PathBuf::new(),
            number: None,
            nested_items: Vec::new(),
        }
    }
}

/// A linked virtual chapter in the `SUMMARY.md`, possibly with nested entries.
///
/// This is roughly the equivalent of `- Some virtual section`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualLink {
    /// The name of the chapter.
    pub name: String,
    /// The section number, if this virtual chapter is in the numbered section.
    pub number: Option<SectionNumber>,
    /// Any nested items this virtual chapter may contain.
    pub nested_items: Vec<SummaryItem>,
}

impl VirtualLink {
    /// Create a new virtual link with no nested items.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            number: None,
            nested_items: Vec::new(),
        }
    }
}

impl Default for VirtualLink {
    fn default() -> Self {
        Self {
            name: String::new(),
            number: None,
            nested_items: Vec::new(),
        }
    }
}

/// An entry in the `SUMMARY.md` which could be either a `Link`, a
/// `VirtualLink` or separator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SummaryItem {
    /// A link to a chapter.
    Link(Link),
    /// A link to a virtual chapter.
    VirtualLink(VirtualLink),
    /// A separator (`---`).
    Separator,
}

impl SummaryItem {
    fn maybe_link_mut(&mut self) -> Option<&mut Link> {
        match *self {
            SummaryItem::Link(ref mut l) => Some(l),
            _ => None,
        }
    }

    fn maybe_virtual_link_mut(&mut self) -> Option<&mut VirtualLink> {
        match *self {
            SummaryItem::VirtualLink(ref mut l) => Some(l),
            _ => None,
        }
    }
}

impl From<Link> for SummaryItem {
    fn from(other: Link) -> Self {
        SummaryItem::Link(other)
    }
}

impl From<VirtualLink> for SummaryItem {
    fn from(other: VirtualLink) -> Self {
        SummaryItem::VirtualLink(other)
    }
}

/// A recursive descent (-ish) parser for a `SUMMARY.md`.
///
///
/// # Grammar
///
/// The `SUMMARY.md` file has a grammar which looks something like this:
///
/// ```text
/// summary           ::= title prefix_chapters numbered_chapters
/// suffix_chapters
/// title             ::= "# " TEXT
///                     | EPSILON
/// prefix_chapters   ::= item*
/// suffix_chapters   ::= item*
/// numbered_chapters ::= dotted_item+
/// dotted_item       ::= INDENT* DOT_POINT item
/// item              ::= link
///                     | virtual_link
///                     | separator
/// link              ::= "[" TEXT "]" "(" TEXT ")"
/// virtual_link      ::= "[" TEXT "]" "()"
/// separator         ::= "---"
/// DOT_POINT         ::= "-"
///                     | "*"
/// ```
///
/// > **Note:** the `TEXT` terminal is "normal" text, and should (roughly)
/// > match the following regex: "[^<>\n[]]+".
struct SummaryParser<'a> {
    src: &'a str,
    stream: pulldown_cmark::Parser<'a>,
}

/// Reads `Events` from the provided stream until the corresponding
/// `Event::End` is encountered which matches the `$delimiter` pattern.
///
/// This is the equivalent of doing
/// `$stream.take_while(|e| e != $delimeter).collect()` but it allows you to
/// use pattern matching and you won't get errors because `take_while()`
/// moves `$stream` out of self.
macro_rules! collect_events {
    ($stream:expr, start $delimiter:pat) => {
        collect_events!($stream, Event::Start($delimiter))
    };
    ($stream:expr, end $delimiter:pat) => {
        collect_events!($stream, Event::End($delimiter))
    };
    ($stream:expr, $delimiter:pat) => {
        {
            let mut events = Vec::new();

            loop {
                let event = $stream.next();
                trace!("Next event: {:?}", event);

                match event {
                    Some($delimiter) => break,
                    Some(other) => events.push(other),
                    None => {
                        debug!("Reached end of stream without finding the closing pattern, {}", stringify!($delimiter));
                        break;
                    }
                }
            }

            events
        }
    }
}

impl<'a> SummaryParser<'a> {
    fn new(text: &str) -> SummaryParser {
        let pulldown_parser = pulldown_cmark::Parser::new(text);

        SummaryParser {
            src: text,
            stream: pulldown_parser,
        }
    }

    /// Get the current line and column to give the user more useful error
    /// messages.
    fn current_location(&self) -> (usize, usize) {
        let byte_offset = self.stream.get_offset();

        let previous_text = self.src[..byte_offset].as_bytes();
        let line = Memchr::new(b'\n', previous_text).count() + 1;
        let start_of_line = memchr::memrchr(b'\n', previous_text).unwrap_or(0);
        let col = self.src[start_of_line..byte_offset].chars().count();

        (line, col)
    }

    /// Parse the text the `SummaryParser` was created with.
    fn parse(mut self) -> Result<Summary> {
        let title = self.parse_title();

        let prefix_chapters = self.parse_affix(true)
            .chain_err(|| "There was an error parsing the prefix chapters")?;
        let numbered_chapters = self.parse_numbered()
            .chain_err(|| "There was an error parsing the numbered chapters")?;
        let suffix_chapters = self.parse_affix(false)
            .chain_err(|| "There was an error parsing the suffix chapters")?;

        Ok(Summary {
            title,
            prefix_chapters,
            numbered_chapters,
            suffix_chapters,
        })
    }

    /// Parse the affix chapters. This expects the first event (start of
    /// paragraph) to have already been consumed by the previous parser.
    fn parse_affix(&mut self, is_prefix: bool) -> Result<Vec<SummaryItem>> {
        let mut items = Vec::new();
        debug!(
            "Parsing {} items",
            if is_prefix { "prefix" } else { "suffix" }
        );

        loop {
            match self.next_event() {
                Some(Event::Start(Tag::List(..))) => {
                    if is_prefix {
                        // we've finished prefix chapters and are at the start
                        // of the numbered section.
                        break;
                    } else {
                        bail!(self.parse_error("Suffix chapters cannot be followed by a list"));
                    }
                }
                Some(Event::Start(Tag::Link(href, _))) => items.push(self.parse_linklike(href)),
                Some(Event::Start(Tag::Rule)) => items.push(SummaryItem::Separator),
                Some(_) => {}
                None => break,
            }
        }

        Ok(items)
    }

    fn parse_linklike(&mut self, href: Cow<'a, str>) -> SummaryItem {
        let name = {
            let link_content = collect_events!(self.stream, end Tag::Link(..));
            stringify_events(link_content)
        };

        if href.is_empty() {
            let link = VirtualLink::new(name);
            SummaryItem::VirtualLink(link)
        } else {
            let link = Link::new(name, href.into_owned());
            SummaryItem::Link(link)
        }
    }

    /// Parse the numbered chapters. This assumes the opening list tag has
    /// already been consumed by a previous parser.
    fn parse_numbered(&mut self) -> Result<Vec<SummaryItem>> {
        let mut items = Vec::new();
        let root_number = SectionNumber::default();

        // we need to do this funny loop-match-if-let dance because a rule will
        // close off any currently running list. Therefore we try to read the
        // list items before the rule, then if we encounter a rule we'll add a
        // separator and try to resume parsing numbered chapters if we start a
        // list immediately afterwards.
        //
        // If you can think of a better way to do this then please make a PR :)

        loop {
            let mut bunch_of_items = self.parse_nested_numbered(&root_number)?;

            // if we've resumed after something like a rule the root sections
            // will be numbered from 1. We need to manually go back and update
            // them
            update_section_numbers(&mut bunch_of_items, 0, items.len() as u32);
            items.extend(bunch_of_items);

            match self.next_event() {
                Some(Event::Start(Tag::Paragraph)) => {
                    // we're starting the suffix chapters
                    break;
                }
                Some(Event::Start(other_tag)) => {
                    if other_tag == Tag::Rule {
                        items.push(SummaryItem::Separator);
                    }
                    trace!("Skipping contents of {:?}", other_tag);

                    // Skip over the contents of this tag
                    while let Some(event) = self.next_event() {
                        if event == Event::End(other_tag.clone()) {
                            break;
                        }
                    }

                    if let Some(Event::Start(Tag::List(..))) = self.next_event() {
                        continue;
                    } else {
                        break;
                    }
                }
                Some(_) => {
                    // something else... ignore
                    continue;
                }
                None => {
                    // EOF, bail...
                    break;
                }
            }
        }

        Ok(items)
    }

    fn next_event(&mut self) -> Option<Event<'a>> {
        let next = self.stream.next();
        trace!("Next event: {:?}", next);

        next
    }

    fn parse_nested_numbered(&mut self, parent: &SectionNumber) -> Result<Vec<SummaryItem>> {
        debug!("Parsing numbered chapters at level {}", parent);
        let mut items = Vec::new();

        loop {
            match self.next_event() {
                Some(Event::Start(Tag::Item)) => {
                    let item = self.parse_nested_item(parent, items.len())?;
                    items.push(item);
                }
                Some(Event::Start(Tag::List(..))) => {
                    // recurse to parse the nested list
                    let (_, last_item) = get_last_linklike(&mut items)?;

                    match *last_item {
                        SummaryItem::Link(ref mut last_link) => {
                            let last_item_number = last_link
                                .number
                                .as_ref()
                                .expect("All numbered chapters have numbers");

                            let sub_items = self.parse_nested_numbered(last_item_number)?;

                            last_link.nested_items = sub_items;
                        },
                        SummaryItem::VirtualLink(ref mut last_link) => {
                            let last_item_number = last_link
                                .number
                                .as_ref()
                                .expect("All numbered chapters have numbers");

                            let sub_items = self.parse_nested_numbered(last_item_number)?;

                            last_link.nested_items = sub_items;
                        },
                        SummaryItem::Separator => unreachable!(),
                    };
                }
                Some(Event::End(Tag::List(..))) => break,
                Some(_) => {}
                None => break,
            }
        }

        Ok(items)
    }

    fn parse_nested_item(
        &mut self,
        parent: &SectionNumber,
        num_existing_items: usize,
    ) -> Result<SummaryItem> {
        loop {
            match self.next_event() {
                Some(Event::Start(Tag::Paragraph)) => continue,
                Some(Event::Start(Tag::Link(href, _))) => {
                    let mut item = self.parse_linklike(href);

                    let mut number = parent.clone();
                    number.0.push(num_existing_items as u32 + 1);

                    match item {
                        SummaryItem::Link(ref mut link) => {
                            trace!(
                                "Found chapter: {} {} ({})",
                                number,
                                link.name,
                                link.location.display(),
                            );

                            link.number = Some(number);
                        },
                        SummaryItem::VirtualLink(ref mut link) => {
                            trace!(
                                "Found virtual chapter: {} {}",
                                number,
                                link.name,
                            );

                            link.number = Some(number);
                        },
                        SummaryItem::Separator => panic!(),
                    }

                    return Ok(item);
                }
                other => {
                    warn!("Expected a start of a link, actually got {:?}", other);
                    bail!(self.parse_error(
                        "The link items for nested chapters must only contain a hyperlink"
                    ));
                }
            }
        }
    }

    fn parse_error<D: Display>(&self, msg: D) -> Error {
        let (line, col) = self.current_location();

        ErrorKind::ParseError(line, col, msg.to_string()).into()
    }

    /// Try to parse the title line.
    fn parse_title(&mut self) -> Option<String> {
        if let Some(Event::Start(Tag::Header(1))) = self.next_event() {
            debug!("Found a h1 in the SUMMARY");

            let tags = collect_events!(self.stream, end Tag::Header(1));
            Some(stringify_events(tags))
        } else {
            None
        }
    }
}

fn update_section_numbers(sections: &mut [SummaryItem], level: usize, by: u32) {
    for section in sections {
        if let SummaryItem::Link(ref mut link) = *section {
            if let Some(ref mut number) = link.number {
                number.0[level] += by;
            }

            update_section_numbers(&mut link.nested_items, level, by);
        }
    }
}

/// Gets a pointer to the last `Link` or `VirtualLink` in a list of
/// `SummaryItem`s, and its index.
fn get_last_linklike(items: &mut [SummaryItem]) -> Result<(usize, &mut SummaryItem)> {
    items
        .iter_mut()
        .enumerate()
        .filter(|&(_, ref item)| {
            match **item {
                SummaryItem::Link(_) => true,
                SummaryItem::VirtualLink(_) => true,
                SummaryItem::Separator => false,
            }
        })
        .rev()
        .next()
        .ok_or_else(|| {
            "Unable to get last Link or VirtualLink because the list of SummaryItems doesn't contain any"
                .into()
        })
}

/// Removes the styling from a list of Markdown events and returns just the
/// plain text.
fn stringify_events(events: Vec<Event>) -> String {
    events
        .into_iter()
        .filter_map(|t| match t {
            Event::Text(text) => Some(text.into_owned()),
            _ => None,
        })
        .collect()
}

/// A section number like "1.2.3", basically just a newtype'd `Vec<u32>` with
/// a pretty `Display` impl.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct SectionNumber(pub Vec<u32>);

impl Display for SectionNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "0")
        } else {
            for item in &self.0 {
                write!(f, "{}.", item)?;
            }
            Ok(())
        }
    }
}

impl Deref for SectionNumber {
    type Target = Vec<u32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SectionNumber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<u32> for SectionNumber {
    fn from_iter<I: IntoIterator<Item = u32>>(it: I) -> Self {
        SectionNumber(it.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_number_has_correct_dotted_representation() {
        let inputs = vec![
            (vec![0], "0."),
            (vec![1, 3], "1.3."),
            (vec![1, 2, 3], "1.2.3."),
        ];

        for (input, should_be) in inputs {
            let section_number = SectionNumber(input).to_string();
            assert_eq!(section_number, should_be);
        }
    }

    #[test]
    fn parse_initial_title() {
        let src = "# Summary";
        let should_be = String::from("Summary");

        let mut parser = SummaryParser::new(src);
        let got = parser.parse_title().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_title_with_styling() {
        let src = "# My **Awesome** Summary";
        let should_be = String::from("My Awesome Summary");

        let mut parser = SummaryParser::new(src);
        let got = parser.parse_title().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn convert_markdown_events_to_a_string() {
        let src = "Hello *World*, `this` is some text [and a link](./path/to/link)";
        let should_be = "Hello World, this is some text and a link";

        let events = pulldown_cmark::Parser::new(src).collect();
        let got = stringify_events(events);

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_some_prefix_items() {
        let src = "[First](./first.md)\n[Second](./second.md)\n";
        let mut parser = SummaryParser::new(src);

        let should_be = vec![
            SummaryItem::Link(Link {
                name: String::from("First"),
                location: PathBuf::from("./first.md"),
                ..Default::default()
            }),
            SummaryItem::Link(Link {
                name: String::from("Second"),
                location: PathBuf::from("./second.md"),
                ..Default::default()
            }),
        ];

        let _ = parser.stream.next(); // step past first event
        let got = parser.parse_affix(true).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_prefix_items_with_a_separator() {
        let src = "[First](./first.md)\n\n---\n\n[Second](./second.md)\n";
        let mut parser = SummaryParser::new(src);

        let _ = parser.stream.next(); // step past first event
        let got = parser.parse_affix(true).unwrap();

        assert_eq!(got.len(), 3);
        assert_eq!(got[1], SummaryItem::Separator);
    }

    #[test]
    fn suffix_items_cannot_be_followed_by_a_list() {
        let src = "[First](./first.md)\n- [Second](./second.md)\n";
        let mut parser = SummaryParser::new(src);

        let _ = parser.stream.next(); // step past first event
        let got = parser.parse_affix(false);

        assert!(got.is_err());
    }

    #[test]
    fn parse_a_link() {
        let src = "[Chapter](./chapter.md)";
        let should_be = SummaryItem::Link(
            Link {
                name: String::from("Chapter"),
                location: PathBuf::from("./chapter.md"),
                ..Default::default()
            }
        );

        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next(); // skip past start of paragraph

        let href = match parser.stream.next() {
            Some(Event::Start(Tag::Link(href, _))) => href,
            other => panic!("Unreachable, {:?}", other),
        };

        let got = parser.parse_linklike(href);
        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_a_virtual_link() {
        let src = "[Virtual chapter]()";
        let should_be = SummaryItem::VirtualLink(
            VirtualLink {
                name: String::from("Virtual chapter"),
                ..Default::default()
            }
        );

        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next(); // skip past start of paragraph

        let href = match parser.stream.next() {
            Some(Event::Start(Tag::Link(href, _))) => href,
            other => panic!("Unreachable, {:?}", other),
        };

        let got = parser.parse_linklike(href);
        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_a_numbered_chapter() {
        let src = "- [First](./first.md)\n";
        let link = Link {
            name: String::from("First"),
            location: PathBuf::from("./first.md"),
            number: Some(SectionNumber(vec![1])),
            ..Default::default()
        };
        let should_be = vec![SummaryItem::Link(link)];

        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next();

        let got = parser.parse_numbered().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_nested_numbered_chapters() {
        let src = "- [First](./first.md)\n  - [Nested](./nested.md)\n- [Second](./second.md)";

        let should_be = vec![
            SummaryItem::Link(Link {
                name: String::from("First"),
                location: PathBuf::from("./first.md"),
                number: Some(SectionNumber(vec![1])),
                nested_items: vec![
                    SummaryItem::Link(Link {
                        name: String::from("Nested"),
                        location: PathBuf::from("./nested.md"),
                        number: Some(SectionNumber(vec![1, 1])),
                        nested_items: Vec::new(),
                    }),
                ],
            }),
            SummaryItem::Link(Link {
                name: String::from("Second"),
                location: PathBuf::from("./second.md"),
                number: Some(SectionNumber(vec![2])),
                nested_items: Vec::new(),
            }),
        ];

        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next();

        let got = parser.parse_numbered().unwrap();

        assert_eq!(got, should_be);
    }

    /// This test ensures the book will continue to pass because it breaks the
    /// `SUMMARY.md` up using level 2 headers ([example]).
    ///
    /// [example]: https://github.com/rust-lang/book/blob/2c942dc094f4ddcdc7aba7564f80782801197c99/second-edition/src/SUMMARY.md#basic-rust-literacy
    #[test]
    fn can_have_a_subheader_between_nested_items() {
        let src = "- [First](./first.md)\n\n## Subheading\n\n- [Second](./second.md)\n";
        let should_be = vec![
            SummaryItem::Link(Link {
                name: String::from("First"),
                location: PathBuf::from("./first.md"),
                number: Some(SectionNumber(vec![1])),
                nested_items: Vec::new(),
            }),
            SummaryItem::Link(Link {
                name: String::from("Second"),
                location: PathBuf::from("./second.md"),
                number: Some(SectionNumber(vec![2])),
                nested_items: Vec::new(),
            }),
        ];

        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next();

        let got = parser.parse_numbered().unwrap();

        assert_eq!(got, should_be);
    }
}
