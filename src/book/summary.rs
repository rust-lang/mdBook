use crate::errors::*;
use memchr::{self, Memchr};
use pulldown_cmark::{self, Event, Tag};
use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

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
/// **Part Title:** An optional title for the next collect of numbered chapters. The numbered
/// chapters can be broken into as many parts as desired.
///
/// **Numbered Chapter:** Numbered chapters are the main content of the book,
/// they
/// will be numbered and can be nested, resulting in a nice hierarchy (chapters,
/// sub-chapters, etc.)
///
/// ```markdown
/// # Title of Part
///
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
    /// The main numbered chapters of the book, broken into one or more possibly named parts.
    pub numbered_chapters: Vec<SummaryItem>,
    /// Items which come after the main document (e.g. a conclusion).
    pub suffix_chapters: Vec<SummaryItem>,
}

impl Summary {
    /// Create a summary from the book's sources directory.
    ///
    /// Each file is imported as a book chapter.
    /// Each folder is imported as a book chapter and must contain
    /// a `README.md` file defining the chapter's title and content.
    /// Any file/folder inside the directory is imported as a sub-chapter.
    /// The file/folder name is used to compose the chapter's link.
    ///
    /// Chapters are added to the book in alphabetical order, using the file/folder name.
    /// If a `src_dir/README.md` file is present, it is included as a prefix chapter.
    pub fn from_sources<P: AsRef<Path>>(src_dir: P) -> std::io::Result<Summary> {
        let mut summary = Summary {
            title: None,
            prefix_chapters: Vec::new(),
            numbered_chapters: Vec::new(),
            suffix_chapters: Vec::new(),
        };

        // Checks if the given path must be considered.
        fn include_path(path: &Path) -> bool {
            if let Some(name) = path.file_name() {
                if name == "README.md" || name == "SUMMARY.md" {
                    return false;
                }
            }

            true
        }

        // Read a directory recursively and return the found summary items.
        fn read_dir<P: AsRef<Path>>(dir: P) -> std::io::Result<Vec<SummaryItem>> {
            let mut links = Vec::new();

            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let entry_path = entry.path();

                if include_path(&entry_path) {
                    let metadata = std::fs::metadata(&entry_path)?;

                    if metadata.is_file() {
                        links.push(Link::new_unnamed(entry_path))
                    } else {
                        let chapter_path = entry_path.join("README.md");
                        if chapter_path.is_file() {
                            let mut link = Link::new_unnamed(chapter_path);
                            link.nested_items = read_dir(entry_path)?;
                            links.push(link)
                        }
                    }
                }
            }

            // Items are sorted by name.
            links.sort_by(|a, b| {
                a.location
                    .as_ref()
                    .unwrap()
                    .cmp(b.location.as_ref().unwrap())
            });

            Ok(links.into_iter().map(SummaryItem::Link).collect())
        }

        // Associate the correct section number to each summary item.
        fn number_items(items: &mut [SummaryItem], number: &[u32]) {
            let mut n = 1;
            for item in items {
                if let SummaryItem::Link(link) = item {
                    let mut entry_number = number.to_vec();
                    entry_number.push(n);
                    n += 1;
                    number_items(&mut link.nested_items, &entry_number);
                    link.number = Some(SectionNumber(entry_number))
                }
            }
        }

        // Special case for the `src/README.md` file.
        let root_readme_path = src_dir.as_ref().join("README.md");
        if root_readme_path.is_file() {
            let link = Link::new_unnamed(root_readme_path);
            summary.prefix_chapters.push(SummaryItem::Link(link))
        }

        summary.numbered_chapters = read_dir(src_dir)?;
        number_items(&mut summary.numbered_chapters, &[]);

        Ok(summary)
    }
}

/// A struct representing an entry in the `SUMMARY.md`, possibly with nested
/// entries.
///
/// This is roughly the equivalent of `[Some section](./path/to/file.md)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// The name of the chapter.
    pub name: Option<String>,
    /// The location of the chapter's source file, taking the book's `src`
    /// directory as the root.
    pub location: Option<PathBuf>,
    /// The section number, if this chapter is in the numbered section.
    pub number: Option<SectionNumber>,
    /// Any nested items this chapter may contain.
    pub nested_items: Vec<SummaryItem>,
}

impl Link {
    /// Create a new link with no nested items.
    pub fn new<S: Into<String>, P: AsRef<Path>>(name: S, location: P) -> Link {
        Link {
            name: Some(name.into()),
            location: Some(location.as_ref().to_path_buf()),
            number: None,
            nested_items: Vec::new(),
        }
    }

    /// Create a new unnamed link with no nested items.
    pub fn new_unnamed<P: AsRef<Path>>(location: P) -> Link {
        Link {
            name: None,
            location: Some(location.as_ref().to_path_buf()),
            number: None,
            nested_items: Vec::new(),
        }
    }
}

impl Default for Link {
    fn default() -> Self {
        Link {
            name: None,
            location: Some(PathBuf::new()),
            number: None,
            nested_items: Vec::new(),
        }
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "\"{}\"", name)?,
            None => write!(f, "unnamed chapter")?,
        }

        match &self.location {
            Some(location) => write!(f, " ({})", location.display())?,
            None => write!(f, " [draft]")?,
        }

        Ok(())
    }
}

/// An item in `SUMMARY.md` which could be either a separator or a `Link`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SummaryItem {
    /// A link to a chapter.
    Link(Link),
    /// A separator (`---`).
    Separator,
    /// A part title.
    PartTitle(String),
}

impl SummaryItem {
    fn maybe_link_mut(&mut self) -> Option<&mut Link> {
        match *self {
            SummaryItem::Link(ref mut l) => Some(l),
            _ => None,
        }
    }
}

impl From<Link> for SummaryItem {
    fn from(other: Link) -> SummaryItem {
        SummaryItem::Link(other)
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
///                         suffix_chapters
/// title             ::= "# " TEXT
///                     | EPSILON
/// prefix_chapters   ::= item*
/// suffix_chapters   ::= item*
/// numbered_chapters ::= part+
/// part              ::= title dotted_item+
/// dotted_item       ::= INDENT* DOT_POINT item
/// item              ::= link
///                     | separator
/// separator         ::= "---"
/// link              ::= "[" TEXT "]" "(" TEXT ")"
/// DOT_POINT         ::= "-"
///                     | "*"
/// ```
///
/// > **Note:** the `TEXT` terminal is "normal" text, and should (roughly)
/// > match the following regex: "[^<>\n[]]+".
struct SummaryParser<'a> {
    src: &'a str,
    stream: pulldown_cmark::OffsetIter<'a>,
    offset: usize,

    /// We can't actually put an event back into the `OffsetIter` stream, so instead we store it
    /// here until somebody calls `next_event` again.
    back: Option<Event<'a>>,
}

/// Reads `Events` from the provided stream until the corresponding
/// `Event::End` is encountered which matches the `$delimiter` pattern.
///
/// This is the equivalent of doing
/// `$stream.take_while(|e| e != $delimiter).collect()` but it allows you to
/// use pattern matching and you won't get errors because `take_while()`
/// moves `$stream` out of self.
macro_rules! collect_events {
    ($stream:expr,start $delimiter:pat) => {
        collect_events!($stream, Event::Start($delimiter))
    };
    ($stream:expr,end $delimiter:pat) => {
        collect_events!($stream, Event::End($delimiter))
    };
    ($stream:expr, $delimiter:pat) => {{
        let mut events = Vec::new();

        loop {
            let event = $stream.next().map(|(ev, _range)| ev);
            trace!("Next event: {:?}", event);

            match event {
                Some($delimiter) => break,
                Some(other) => events.push(other),
                None => {
                    debug!(
                        "Reached end of stream without finding the closing pattern, {}",
                        stringify!($delimiter)
                    );
                    break;
                }
            }
        }

        events
    }};
}

impl<'a> SummaryParser<'a> {
    fn new(text: &str) -> SummaryParser<'_> {
        let pulldown_parser = pulldown_cmark::Parser::new(text).into_offset_iter();

        SummaryParser {
            src: text,
            stream: pulldown_parser,
            offset: 0,
            back: None,
        }
    }

    /// Get the current line and column to give the user more useful error
    /// messages.
    fn current_location(&self) -> (usize, usize) {
        let previous_text = self.src[..self.offset].as_bytes();
        let line = Memchr::new(b'\n', previous_text).count() + 1;
        let start_of_line = memchr::memrchr(b'\n', previous_text).unwrap_or(0);
        let col = self.src[start_of_line..self.offset].chars().count();

        (line, col)
    }

    /// Parse the text the `SummaryParser` was created with.
    fn parse(mut self) -> Result<Summary> {
        let title = self.parse_title();

        let prefix_chapters = self
            .parse_affix(true)
            .with_context(|| "There was an error parsing the prefix chapters")?;
        let numbered_chapters = self
            .parse_parts()
            .with_context(|| "There was an error parsing the numbered chapters")?;
        let suffix_chapters = self
            .parse_affix(false)
            .with_context(|| "There was an error parsing the suffix chapters")?;

        Ok(Summary {
            title,
            prefix_chapters,
            numbered_chapters,
            suffix_chapters,
        })
    }

    /// Parse the affix chapters.
    fn parse_affix(&mut self, is_prefix: bool) -> Result<Vec<SummaryItem>> {
        let mut items = Vec::new();
        debug!(
            "Parsing {} items",
            if is_prefix { "prefix" } else { "suffix" }
        );

        loop {
            match self.next_event() {
                Some(ev @ Event::Start(Tag::List(..)))
                | Some(ev @ Event::Start(Tag::Heading(1))) => {
                    if is_prefix {
                        // we've finished prefix chapters and are at the start
                        // of the numbered section.
                        self.back(ev);
                        break;
                    } else {
                        bail!(self.parse_error("Suffix chapters cannot be followed by a list"));
                    }
                }
                Some(Event::Start(Tag::Link(_type, href, _title))) => {
                    let link = self.parse_link(href.to_string());
                    items.push(SummaryItem::Link(link));
                }
                Some(Event::Rule) => items.push(SummaryItem::Separator),
                Some(_) => {}
                None => break,
            }
        }

        Ok(items)
    }

    fn parse_parts(&mut self) -> Result<Vec<SummaryItem>> {
        let mut parts = vec![];

        // We want the section numbers to be continues through all parts.
        let mut root_number = SectionNumber::default();
        let mut root_items = 0;

        loop {
            // Possibly match a title or the end of the "numbered chapters part".
            let title = match self.next_event() {
                Some(ev @ Event::Start(Tag::Paragraph)) => {
                    // we're starting the suffix chapters
                    self.back(ev);
                    break;
                }

                Some(Event::Start(Tag::Heading(1))) => {
                    debug!("Found a h1 in the SUMMARY");

                    let tags = collect_events!(self.stream, end Tag::Heading(1));
                    Some(stringify_events(tags))
                }

                Some(ev) => {
                    self.back(ev);
                    None
                }

                None => break, // EOF, bail...
            };

            // Parse the rest of the part.
            let numbered_chapters = self
                .parse_numbered(&mut root_items, &mut root_number)
                .with_context(|| "There was an error parsing the numbered chapters")?;

            if let Some(title) = title {
                parts.push(SummaryItem::PartTitle(title));
            }
            parts.extend(numbered_chapters);
        }

        Ok(parts)
    }

    /// Finishes parsing a link once the `Event::Start(Tag::Link(..))` has been opened.
    fn parse_link(&mut self, href: String) -> Link {
        let href = href.replace("%20", " ");
        let link_content = collect_events!(self.stream, end Tag::Link(..));
        let name = stringify_events(link_content);

        let path = if href.is_empty() {
            None
        } else {
            Some(PathBuf::from(href))
        };

        Link {
            name: Some(name),
            location: path,
            number: None,
            nested_items: Vec::new(),
        }
    }

    /// Parse the numbered chapters.
    fn parse_numbered(
        &mut self,
        root_items: &mut u32,
        root_number: &mut SectionNumber,
    ) -> Result<Vec<SummaryItem>> {
        let mut items = Vec::new();

        // For the first iteration, we want to just skip any opening paragraph tags, as that just
        // marks the start of the list. But after that, another opening paragraph indicates that we
        // have started a new part or the suffix chapters.
        let mut first = true;

        loop {
            match self.next_event() {
                Some(ev @ Event::Start(Tag::Paragraph)) => {
                    if !first {
                        // we're starting the suffix chapters
                        self.back(ev);
                        break;
                    }
                }
                // The expectation is that pulldown cmark will terminate a paragraph before a new
                // heading, so we can always count on this to return without skipping headings.
                Some(ev @ Event::Start(Tag::Heading(1))) => {
                    // we're starting a new part
                    self.back(ev);
                    break;
                }
                Some(ev @ Event::Start(Tag::List(..))) => {
                    self.back(ev);
                    let mut bunch_of_items = self.parse_nested_numbered(root_number)?;

                    // if we've resumed after something like a rule the root sections
                    // will be numbered from 1. We need to manually go back and update
                    // them
                    update_section_numbers(&mut bunch_of_items, 0, *root_items);
                    *root_items += bunch_of_items.len() as u32;
                    items.extend(bunch_of_items);
                }
                Some(Event::Start(other_tag)) => {
                    trace!("Skipping contents of {:?}", other_tag);

                    // Skip over the contents of this tag
                    while let Some(event) = self.next_event() {
                        if event == Event::End(other_tag.clone()) {
                            break;
                        }
                    }
                }
                Some(Event::Rule) => {
                    items.push(SummaryItem::Separator);
                }

                // something else... ignore
                Some(_) => {}

                // EOF, bail...
                None => {
                    break;
                }
            }

            // From now on, we cannot accept any new paragraph opening tags.
            first = false;
        }

        Ok(items)
    }

    /// Push an event back to the tail of the stream.
    fn back(&mut self, ev: Event<'a>) {
        assert!(self.back.is_none());
        trace!("Back: {:?}", ev);
        self.back = Some(ev);
    }

    fn next_event(&mut self) -> Option<Event<'a>> {
        let next = self.back.take().or_else(|| {
            self.stream.next().map(|(ev, range)| {
                self.offset = range.start;
                ev
            })
        });

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
                    // Skip this tag after comment bacause it is not nested.
                    if items.is_empty() {
                        continue;
                    }
                    // recurse to parse the nested list
                    let (_, last_item) = get_last_link(&mut items)?;
                    let last_item_number = last_item
                        .number
                        .as_ref()
                        .expect("All numbered chapters have numbers");

                    let sub_items = self.parse_nested_numbered(last_item_number)?;

                    last_item.nested_items = sub_items;
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
                Some(Event::Start(Tag::Link(_type, href, _title))) => {
                    let mut link = self.parse_link(href.to_string());

                    let mut number = parent.clone();
                    number.0.push(num_existing_items as u32 + 1);
                    trace!("Found chapter: {} {}", number, link);

                    link.number = Some(number);

                    return Ok(SummaryItem::Link(link));
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
        anyhow::anyhow!(
            "failed to parse SUMMARY.md line {}, column {}: {}",
            line,
            col,
            msg
        )
    }

    /// Try to parse the title line.
    fn parse_title(&mut self) -> Option<String> {
        loop {
            match self.next_event() {
                Some(Event::Start(Tag::Heading(1))) => {
                    debug!("Found a h1 in the SUMMARY");

                    let tags = collect_events!(self.stream, end Tag::Heading(1));
                    return Some(stringify_events(tags));
                }
                // Skip a HTML element such as a comment line.
                Some(Event::Html(_)) => {}
                // Otherwise, no title.
                _ => return None,
            }
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

/// Gets a pointer to the last `Link` in a list of `SummaryItem`s, and its
/// index.
fn get_last_link(links: &mut [SummaryItem]) -> Result<(usize, &mut Link)> {
    links
        .iter_mut()
        .enumerate()
        .filter_map(|(i, item)| item.maybe_link_mut().map(|l| (i, l)))
        .rev()
        .next()
        .ok_or_else(||
            anyhow::anyhow!("Unable to get last link because the list of SummaryItems doesn't contain any Links")
            )
}

/// Removes the styling from a list of Markdown events and returns just the
/// plain text.
fn stringify_events(events: Vec<Event<'_>>) -> String {
    events
        .into_iter()
        .filter_map(|t| match t {
            Event::Text(text) | Event::Code(text) => Some(text.into_string()),
            Event::SoftBreak => Some(String::from(" ")),
            _ => None,
        })
        .collect()
}

/// A section number like "1.2.3", basically just a newtype'd `Vec<u32>` with
/// a pretty `Display` impl.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct SectionNumber(pub Vec<u32>);

impl Display for SectionNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
                name: Some(String::from("First")),
                location: Some(PathBuf::from("./first.md")),
                ..Default::default()
            }),
            SummaryItem::Link(Link {
                name: Some(String::from("Second")),
                location: Some(PathBuf::from("./second.md")),
                ..Default::default()
            }),
        ];

        let got = parser.parse_affix(true).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_prefix_items_with_a_separator() {
        let src = "[First](./first.md)\n\n---\n\n[Second](./second.md)\n";
        let mut parser = SummaryParser::new(src);

        let got = parser.parse_affix(true).unwrap();

        assert_eq!(got.len(), 3);
        assert_eq!(got[1], SummaryItem::Separator);
    }

    #[test]
    fn suffix_items_cannot_be_followed_by_a_list() {
        let src = "[First](./first.md)\n- [Second](./second.md)\n";
        let mut parser = SummaryParser::new(src);

        let got = parser.parse_affix(false);

        assert!(got.is_err());
    }

    #[test]
    fn parse_a_link() {
        let src = "[First](./first.md)";
        let should_be = Link {
            name: Some(String::from("First")),
            location: Some(PathBuf::from("./first.md")),
            ..Default::default()
        };

        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next(); // Discard opening paragraph

        let href = match parser.stream.next() {
            Some((Event::Start(Tag::Link(_type, href, _title)), _range)) => href.to_string(),
            other => panic!("Unreachable, {:?}", other),
        };

        let got = parser.parse_link(href);
        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_a_numbered_chapter() {
        let src = "- [First](./first.md)\n";
        let link = Link {
            name: Some(String::from("First")),
            location: Some(PathBuf::from("./first.md")),
            number: Some(SectionNumber(vec![1])),
            ..Default::default()
        };
        let should_be = vec![SummaryItem::Link(link)];

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_nested_numbered_chapters() {
        let src = "- [First](./first.md)\n  - [Nested](./nested.md)\n- [Second](./second.md)";

        let should_be = vec![
            SummaryItem::Link(Link {
                name: Some(String::from("First")),
                location: Some(PathBuf::from("./first.md")),
                number: Some(SectionNumber(vec![1])),
                nested_items: vec![SummaryItem::Link(Link {
                    name: Some(String::from("Nested")),
                    location: Some(PathBuf::from("./nested.md")),
                    number: Some(SectionNumber(vec![1, 1])),
                    nested_items: Vec::new(),
                })],
            }),
            SummaryItem::Link(Link {
                name: Some(String::from("Second")),
                location: Some(PathBuf::from("./second.md")),
                number: Some(SectionNumber(vec![2])),
                nested_items: Vec::new(),
            }),
        ];

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_numbered_chapters_separated_by_comment() {
        let src = "- [First](./first.md)\n<!-- this is a comment -->\n- [Second](./second.md)";

        let should_be = vec![
            SummaryItem::Link(Link {
                name: Some(String::from("First")),
                location: Some(PathBuf::from("./first.md")),
                number: Some(SectionNumber(vec![1])),
                nested_items: Vec::new(),
            }),
            SummaryItem::Link(Link {
                name: Some(String::from("Second")),
                location: Some(PathBuf::from("./second.md")),
                number: Some(SectionNumber(vec![2])),
                nested_items: Vec::new(),
            }),
        ];

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_titled_parts() {
        let src = "- [First](./first.md)\n- [Second](./second.md)\n\
                   # Title 2\n- [Third](./third.md)\n\t- [Fourth](./fourth.md)";

        let should_be = vec![
            SummaryItem::Link(Link {
                name: Some(String::from("First")),
                location: Some(PathBuf::from("./first.md")),
                number: Some(SectionNumber(vec![1])),
                nested_items: Vec::new(),
            }),
            SummaryItem::Link(Link {
                name: Some(String::from("Second")),
                location: Some(PathBuf::from("./second.md")),
                number: Some(SectionNumber(vec![2])),
                nested_items: Vec::new(),
            }),
            SummaryItem::PartTitle(String::from("Title 2")),
            SummaryItem::Link(Link {
                name: Some(String::from("Third")),
                location: Some(PathBuf::from("./third.md")),
                number: Some(SectionNumber(vec![3])),
                nested_items: vec![SummaryItem::Link(Link {
                    name: Some(String::from("Fourth")),
                    location: Some(PathBuf::from("./fourth.md")),
                    number: Some(SectionNumber(vec![3, 1])),
                    nested_items: Vec::new(),
                })],
            }),
        ];

        let mut parser = SummaryParser::new(src);
        let got = parser.parse_parts().unwrap();

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
                name: Some(String::from("First")),
                location: Some(PathBuf::from("./first.md")),
                number: Some(SectionNumber(vec![1])),
                nested_items: Vec::new(),
            }),
            SummaryItem::Link(Link {
                name: Some(String::from("Second")),
                location: Some(PathBuf::from("./second.md")),
                number: Some(SectionNumber(vec![2])),
                nested_items: Vec::new(),
            }),
        ];

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn an_empty_link_location_is_a_draft_chapter() {
        let src = "- [Empty]()\n";
        let mut parser = SummaryParser::new(src);

        let got = parser.parse_numbered(&mut 0, &mut SectionNumber::default());
        let should_be = vec![SummaryItem::Link(Link {
            name: Some(String::from("Empty")),
            location: None,
            number: Some(SectionNumber(vec![1])),
            nested_items: Vec::new(),
        })];

        assert!(got.is_ok());
        assert_eq!(got.unwrap(), should_be);
    }

    /// Regression test for https://github.com/rust-lang/mdBook/issues/779
    /// Ensure section numbers are correctly incremented after a horizontal separator.
    #[test]
    fn keep_numbering_after_separator() {
        let src =
            "- [First](./first.md)\n---\n- [Second](./second.md)\n---\n- [Third](./third.md)\n";
        let should_be = vec![
            SummaryItem::Link(Link {
                name: Some(String::from("First")),
                location: Some(PathBuf::from("./first.md")),
                number: Some(SectionNumber(vec![1])),
                nested_items: Vec::new(),
            }),
            SummaryItem::Separator,
            SummaryItem::Link(Link {
                name: Some(String::from("Second")),
                location: Some(PathBuf::from("./second.md")),
                number: Some(SectionNumber(vec![2])),
                nested_items: Vec::new(),
            }),
            SummaryItem::Separator,
            SummaryItem::Link(Link {
                name: Some(String::from("Third")),
                location: Some(PathBuf::from("./third.md")),
                number: Some(SectionNumber(vec![3])),
                nested_items: Vec::new(),
            }),
        ];

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_eq!(got, should_be);
    }

    /// Regression test for https://github.com/rust-lang/mdBook/issues/1218
    /// Ensure chapter names spread across multiple lines have spaces between all the words.
    #[test]
    fn add_space_for_multi_line_chapter_names() {
        let src = "- [Chapter\ntitle](./chapter.md)";
        let should_be = vec![SummaryItem::Link(Link {
            name: Some(String::from("Chapter title")),
            location: Some(PathBuf::from("./chapter.md")),
            number: Some(SectionNumber(vec![1])),
            nested_items: Vec::new(),
        })];

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn allow_space_in_link_destination() {
        let src = "- [test1](./test%20link1.md)\n- [test2](<./test link2.md>)";
        let should_be = vec![
            SummaryItem::Link(Link {
                name: Some(String::from("test1")),
                location: Some(PathBuf::from("./test link1.md")),
                number: Some(SectionNumber(vec![1])),
                nested_items: Vec::new(),
            }),
            SummaryItem::Link(Link {
                name: Some(String::from("test2")),
                location: Some(PathBuf::from("./test link2.md")),
                number: Some(SectionNumber(vec![2])),
                nested_items: Vec::new(),
            }),
        ];
        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn skip_html_comments() {
        let src = r#"<!--
# Title - En
-->
# Title - Local

<!--
[Prefix 00-01 - En](ch00-01.md)
[Prefix 00-02 - En](ch00-02.md)
-->
[Prefix 00-01 - Local](ch00-01.md)
[Prefix 00-02 - Local](ch00-02.md)

<!--
## Section Title - En
-->
## Section Title - Localized

<!--
- [Ch 01-00 - En](ch01-00.md)
    - [Ch 01-01 - En](ch01-01.md)
    - [Ch 01-02 - En](ch01-02.md)
-->
- [Ch 01-00 - Local](ch01-00.md)
    - [Ch 01-01 - Local](ch01-01.md)
    - [Ch 01-02 - Local](ch01-02.md)

<!--
- [Ch 02-00 - En](ch02-00.md)
-->
- [Ch 02-00 - Local](ch02-00.md)

<!--
[Appendix A - En](appendix-01.md)
[Appendix B - En](appendix-02.md)
-->`
[Appendix A - Local](appendix-01.md)
[Appendix B - Local](appendix-02.md)
"#;

        let mut parser = SummaryParser::new(src);

        // ---- Title ----
        let title = parser.parse_title();
        assert_eq!(title, Some(String::from("Title - Local")));

        // ---- Prefix Chapters ----

        let new_affix_item = |name, location| {
            SummaryItem::Link(Link {
                name: Some(String::from(name)),
                location: Some(PathBuf::from(location)),
                ..Default::default()
            })
        };

        let should_be = vec![
            new_affix_item("Prefix 00-01 - Local", "ch00-01.md"),
            new_affix_item("Prefix 00-02 - Local", "ch00-02.md"),
        ];

        let got = parser.parse_affix(true).unwrap();
        assert_eq!(got, should_be);

        // ---- Numbered Chapters ----

        let new_numbered_item = |name, location, numbers: &[u32], nested_items| {
            SummaryItem::Link(Link {
                name: Some(String::from(name)),
                location: Some(PathBuf::from(location)),
                number: Some(SectionNumber(numbers.to_vec())),
                nested_items,
            })
        };

        let ch01_nested = vec![
            new_numbered_item("Ch 01-01 - Local", "ch01-01.md", &[1, 1], vec![]),
            new_numbered_item("Ch 01-02 - Local", "ch01-02.md", &[1, 2], vec![]),
        ];

        let should_be = vec![
            new_numbered_item("Ch 01-00 - Local", "ch01-00.md", &[1], ch01_nested),
            new_numbered_item("Ch 02-00 - Local", "ch02-00.md", &[2], vec![]),
        ];
        let got = parser.parse_parts().unwrap();
        assert_eq!(got, should_be);

        // ---- Suffix Chapters ----

        let should_be = vec![
            new_affix_item("Appendix A - Local", "appendix-01.md"),
            new_affix_item("Appendix B - Local", "appendix-02.md"),
        ];

        let got = parser.parse_affix(false).unwrap();
        assert_eq!(got, should_be);
    }
}
