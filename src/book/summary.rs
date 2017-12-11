use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use pulldown_cmark::{self, Event, Tag};
use errors::*;


/// Parse the text from a `SUMMARY.md` file into a sort of "recipe" to be
/// used when loading a book from disk.
///
/// # Summary Format
///
/// **Title:** It's common practice to begin with a title, generally
/// "# Summary". But it is not mandatory, the parser just ignores it. So you
/// can too if you feel like it.
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
/// You can either use - or * to indicate a numbered chapter.
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

/// A struct representing an entry in the `SUMMARY.md`, possibly with nested
/// entries.
///
/// This is roughly the equivalent of `[Some section](./path/to/file.md)`.
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
    pub fn new<S: Into<String>, P: AsRef<Path>>(name: S, location: P) -> Link {
        Link {
            name: name.into(),
            location: location.as_ref().to_path_buf(),
            number: None,
            nested_items: Vec::new(),
        }
    }
}

impl Default for Link {
    fn default() -> Self {
        Link {
            name: String::new(),
            location: PathBuf::new(),
            number: None,
            nested_items: Vec::new(),
        }
    }
}

/// An item in `SUMMARY.md` which could be either a separator or a `Link`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SummaryItem {
    /// A link to a chapter.
    Link(Link),
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
}

impl From<Link> for SummaryItem {
    fn from(other: Link) -> SummaryItem {
        SummaryItem::Link(other)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    Begin,
    PrefixChapters,
    /// Numbered chapters, including the nesting level.
    NumberedChapters(u32),
    SuffixChapters,
    End,
}

/// A state machine parser for parsing a `SUMMARY.md` file.
///
/// The parser has roughly 5 states,
///
/// - **Begin:** the initial state
/// - **Prefix Chapters:** Parsing the prefix chapters
/// - **Numbered Chapters:** Parsing the numbered chapters, using a `usize` to
///   indicate the nesting level (because chapters can have sub-chapters)
/// - **Suffix Chapters:** pretty much identical to the Prefix Chapters
/// - **End:** The final state
///
/// The `parse()` method then continually invokes `step()` until it reaches the
/// `End` state. Parsing is guaranteed to (eventually) finish because the next
/// `Event` is read from the underlying `pulldown_cmark::Parser` and passed
/// into the current state's associated method.
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
    stream: pulldown_cmark::Parser<'a>,
    summary: Summary,
    state: State,
}

/// Reads `Events` from the provided stream until the corresponding
/// `Event::End` is encountered which matches the `$delimiter` pattern.
///
/// This is the equivalent of doing
/// `$stream.take_while(|e| e != $delimeter).collect()` but it allows you to
/// use pattern matching and you won't get errors because `take_while()`
/// moves `$stream` out of self.
macro_rules! collect_events {
    ($stream:expr, $delimiter:pat) => {
        {
            let mut events = Vec::new();

            loop {
                let event = $stream.next();
                trace!("Next event: {:?}", event);
                match event {
                    Some(Event::End($delimiter)) => break,
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
        let intermediate_summary = Summary::default();

        SummaryParser {
            stream: pulldown_parser,
            summary: intermediate_summary,
            state: State::Begin,
        }
    }

    /// Parse the text the `SummaryParser` was created with.
    fn parse(mut self) -> Result<Summary> {
        self.summary.title = self.parse_title();

        if let Some(ref title) = self.summary.title {
            debug!("[*] Title is {:?}", title);
        }

        while self.state != State::End {
            self.step()?;
        }

        Ok(self.summary)
    }

    fn step(&mut self) -> Result<()> {
        if let Some(next_event) = self.stream.next() {
            trace!("[*] Current state: {:?}, next event: {:?}", self.state, next_event);

            match self.state {
                State::Begin => self.step_start(next_event)?,
                State::PrefixChapters | State::SuffixChapters => self.step_affix(next_event)?,
                State::NumberedChapters(_) => self.step_numbered(next_event)?,
                State::End => {},
            }
        } else {
            trace!("[*] Reached end of SUMMARY.md");
            self.state = State::End;
        }

        Ok(())
    }

    /// The very first state, we should see a `Begin Paragraph` token or
    /// it's an error...
    fn step_start(&mut self, event: Event<'a>) -> Result<()> {
        match event {
            Event::Start(Tag::Paragraph) => self.state = State::PrefixChapters,
            Event::Start(Tag::List(_)) => self.state = State::NumberedChapters(0),
            other => bail!("Expected a start of paragraph but got {:?}", other),
        }

        Ok(())
    }

    /// Try to step through an "affix" section (recognising prefix and suffix 
    /// chapters).
    ///
    /// If we encounter a link or horizontal line, it'll get added to the 
    /// section. If we encounter a list, we'll either change to 
    /// `State::NumberedChapter` (for prefix) or throw an error (suffix chapters).
    ///
    /// Anything else will be ignored.
    fn step_affix(&mut self, event: Event<'a>) -> Result<()> {

        match event {
            Event::Start(tag) => self.handle_start_tag_in_affix_chapter(tag)?,
            Event::End(Tag::Rule) => {
                debug!("[*] Found an affix chapter separator");
                self.affix_chapter_list().push(SummaryItem::Separator);
            },
            other => {
                trace!("[*] Skipping unexpected token in summary: {:?}", other);
            },
        }

        Ok(())
    }

    /// A helper function to get the `SummaryItem` list we should add items to
    /// when parsing an affix chapter (i.e. prefix or suffix chapters). 
    fn affix_chapter_list(&mut self) -> &mut Vec<SummaryItem> {
        match self.state {
            State::PrefixChapters => &mut self.summary.prefix_chapters,
            State::SuffixChapters => &mut self.summary.suffix_chapters,
            other => panic!("affix_chapter_list() called with invalid state: {:?}", other),
        }
    }

    fn handle_start_tag_in_affix_chapter(&mut self, tag: Tag) -> Result<()> {
        match tag {
                Tag::Link(location, _) => {
                    let content = collect_events!(self.stream, Tag::Link(_, _));
                    let text = stringify_events(content);
                    let link = Link::new(text, location.as_ref());

                    debug!("[*] Found an affix chapter: {:?}", link.name);
                    self.affix_chapter_list().push(SummaryItem::Link(link));
                },
                Tag::List(_) => {
                    match self.state {
                        State::PrefixChapters => {
                            debug!("[*] Changing from prefix chapters to numbered chapters");
                            self.state = State::NumberedChapters(0);
                        },
                        State::SuffixChapters => bail!("Suffix chapters can't be followed by a list"),
                        _ => unreachable!(),
                    }
                },
                other => trace!("[*] Skipping unknown start tag while parsing affix chapters: {:?}", other),
        }

        Ok(())
    }

    /// Parse the numbered chapters.
    ///
    /// If the event is the start of a list item, consume the entire item and
    /// add a new link to the summary with `push_numbered_section`.
    ///
    /// If the event is the start of a new list, bump the nesting level.
    ///
    /// If the event is the end of a list, decrement the nesting level. When
    /// the nesting level would go negative, we've finished the numbered
    /// section and need to parse the suffix section.
    ///
    /// Otherwise, ignore the event.
    fn step_numbered(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Start(Tag::Item) => {
                let it = self.parse_item().chain_err(
                    || "List items should only contain links",
                )?;

                debug!("[*] Found a chapter: {:?} ({})", it.name, it.location.display());
                let section_number = self.push_numbered_section(SummaryItem::Link(it));
                trace!("[*] Section number is {}", section_number);
            },
            Event::End(Tag::Rule) => {
                debug!("[*] Found a numbered chapter separator");
                self.summary.numbered_chapters.push(SummaryItem::Separator);
                self.state = State::NumberedChapters(0);
            },
            Event::Start(Tag::List(_)) => {
                if let State::NumberedChapters(n) = self.state {
                    self.state = State::NumberedChapters(n + 1);
                    trace!("[*] Nesting level increased to {}", n + 1);
                }
            },
            Event::End(Tag::List(_)) => {
                if let State::NumberedChapters(n) = self.state {
                    if n == 0 {
                        trace!("[*] Finished parsing the numbered chapters");
                        self.state = State::SuffixChapters;
                    } else {
                        trace!("[*] Nesting level decreased to {}", n - 1);
                        self.state = State::NumberedChapters(n - 1);
                    }
                }
            },
            Event::End(Tag::Item) => { /* Ignore */ },
            other => {
                trace!("[*] ignoring token: {:?}", other);
            },
        }

        Ok(())
    }

    /// Parse a single item (`[Some Chapter Name](./path/to/chapter.md)`).
    fn parse_item(&mut self) -> Result<Link> {
        let next = self.stream.next();
        trace!("Parsing an item, next event is {:?}", next);

        if let Some(Event::Start(Tag::Link(dest, _))) = next {
            let content = collect_events!(self.stream, Tag::Link(..));

            Ok(Link::new(stringify_events(content), dest.as_ref()))
        } else {
            bail!("Expected a link, got {:?}", next)
        }
    }

    /// Try to parse the title line.
    fn parse_title(&mut self) -> Option<String> {
        if let Some(Event::Start(Tag::Header(1))) = self.stream.next() {
            debug!("[*] Found a h1 in the SUMMARY");

            let tags = collect_events!(self.stream, Tag::Header(1));

            // TODO: How do we deal with headings like "# My **awesome** summary"?
            // for now, I'm just going to scan through and concatenate the
            // Event::Text tags, skipping any styling.
            Some(stringify_events(tags))
        } else {
            None
        }
    }

    /// Push a new section at the end of the current nesting level.
    fn push_numbered_section(&mut self, item: SummaryItem) -> SectionNumber {
        if let State::NumberedChapters(level) = self.state {
            push_item_at_nesting_level(
                &mut self.summary.numbered_chapters,
                item,
                level as usize,
                SectionNumber::default(),
            ).chain_err(|| {
                format!("The parser should always ensure we add the next \
                item at the correct level ({}:{})", module_path!(), line!())
            })
                .unwrap()
        } else {
            // this method should only ever be called when parsing a numbered
            // section, therefore if we ever get here something has gone
            // hideously wrong...
            error!("Calling push_numbered_section() when not in a numbered section");
            error!("Current state: {:?}", self.state);
            error!("Item: {:?}", item);
            error!("Summary:");
            error!("{:#?}", self.summary);
            panic!("Entered unreachable code, this is a bug");
        }
    }
}

/// Given a particular level (e.g. 3), go that many levels down the `Link`'s
/// nested items then append the provided item to the last `Link` in the
/// list.
fn push_item_at_nesting_level(links: &mut Vec<SummaryItem>, mut item: SummaryItem, level: usize, mut section_number: SectionNumber)
    -> Result<SectionNumber> {
    if level == 0 {
        // set the section number, if applicable
        section_number.push(links.len() as u32 + 1);

        if let SummaryItem::Link(ref mut l) = item {
            l.number = Some(section_number.clone());
        }

        links.push(item);
        Ok(section_number)
    } else {
        let (index, last_link) = get_last_link(links).chain_err(|| {
            format!("The list of links needs to be {} levels deeper (current position {})", 
                level, section_number)
        })?;

        section_number.push(index as u32 + 1);
        push_item_at_nesting_level(&mut last_link.nested_items, item, level - 1, section_number)
    }
}

/// Gets a pointer to the last `Link` in a list of `SummaryItem`s, and its
/// index.
fn get_last_link(links: &mut [SummaryItem]) -> Result<(usize, &mut Link)> {
    // TODO: This should probably be integrated into `Link::push_item()`
    links
        .iter_mut()
        .enumerate()
        .filter_map(|(i, item)| item.maybe_link_mut().map(|l| (i, l)))
        .rev()
        .next()
        .ok_or_else(|| "The list of SummaryItems doesn't contain any Links".into())
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
        for item in &self.0 {
            write!(f, "{}.", item)?;
        }
        Ok(())
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
    fn parse_a_single_item() {
        let src = "[A Chapter](./path/to/chapter)";
        let should_be = Link {
            name: String::from("A Chapter"),
            location: PathBuf::from("./path/to/chapter"),
            number: None,
            nested_items: Vec::new(),
        };

        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next(); // skip the opening paragraph tag
        let got = parser.parse_item().unwrap();

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
    fn can_step_past_first_token() {
        let src = "hello world";
        let should_be = State::PrefixChapters;

        let mut parser = SummaryParser::new(src);
        assert_eq!(parser.state, State::Begin);
        parser.step().unwrap();
        assert_eq!(parser.state, should_be);
    }

    #[test]
    fn first_token_must_be_open_paragraph() {
        let src = "hello world";

        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next(); // manually step past the Start Paragraph
        assert!(parser.step().is_err());
    }

    #[test]
    fn can_parse_prefix_chapter_links() {
        let src = "[Hello World](./foo/bar/baz)";
        let should_be = Link {
            name: String::from("Hello World"),
            location: PathBuf::from("./foo/bar/baz"),
            number: None,
            nested_items: Vec::new(),
        };

        let mut parser = SummaryParser::new(src);
        parser.state = State::PrefixChapters;
        assert!(parser.summary.prefix_chapters.is_empty());

        let _ = parser.stream.next(); // manually step past the Start Paragraph
        parser.step().unwrap();

        assert_eq!(parser.summary.prefix_chapters.len(), 1);
        assert_eq!(parser.summary.prefix_chapters[0], SummaryItem::Link(should_be));
        assert_eq!(parser.state, State::PrefixChapters);
    }

    #[test]
    fn can_parse_prefix_chapter_horizontal_rules() {
        let src = "---";
        let should_be = SummaryItem::Separator;

        let mut parser = SummaryParser::new(src);
        parser.state = State::PrefixChapters;
        assert!(parser.summary.prefix_chapters.is_empty());

        let _ = parser.stream.next(); // manually step past the Start Paragraph
        parser.step().unwrap();

        assert_eq!(parser.summary.prefix_chapters.len(), 1);
        assert_eq!(parser.summary.prefix_chapters[0], should_be);
        assert_eq!(parser.state, State::PrefixChapters);
    }

    #[test]
    fn step_from_prefix_chapters_to_numbered() {
        let src = "- foo";

        let mut parser = SummaryParser::new(src);
        parser.state = State::PrefixChapters;

        // let _ = parser.stream.next(); // manually step past the Start Paragraph
        parser.step().unwrap();

        assert_eq!(parser.state, State::NumberedChapters(0));
    }

    #[test]
    fn push_item_onto_empty_link() {
        let root = Link::new("First", "/");
        let mut links = vec![SummaryItem::Link(root)];

        assert_eq!(links[0].maybe_link_mut().unwrap().nested_items.len(), 0);
        let got = push_item_at_nesting_level(&mut links, SummaryItem::Separator, 1, SectionNumber::default()).unwrap();
        assert_eq!(links[0].maybe_link_mut().unwrap().nested_items.len(), 1);
        assert_eq!(*got, vec![1, 1]);
    }

    #[test]
    fn push_item_onto_complex_link() {
        let mut root = Link::new("First", "/first");
        root.nested_items.push(SummaryItem::Separator);

        let mut child = Link::new("Second", "/first/second");
        child.nested_items.push(SummaryItem::Link(
            Link::new("Third", "/first/second/third"),
        ));
        root.nested_items.push(SummaryItem::Link(child));
        root.nested_items.push(SummaryItem::Separator);

        let mut links = vec![SummaryItem::Link(root)];

        // FIXME: This crap for getting a deeply nested member is just plain ugly :(
        assert_eq!(links[0].maybe_link_mut().unwrap()
            .nested_items[1].maybe_link_mut()
            .unwrap()
            .nested_items[0].maybe_link_mut()
            .unwrap()
            .nested_items.len() , 0);
        let got = push_item_at_nesting_level(
            &mut links,
            SummaryItem::Link(Link::new("Dummy", "")),
            3,
            SectionNumber::default(),
        ).unwrap();
        assert_eq!(links[0].maybe_link_mut().unwrap()
            .nested_items[1].maybe_link_mut()
            .unwrap()
            .nested_items[0].maybe_link_mut()
            .unwrap()
            .nested_items.len() , 1);
        println!("{:#?}", links);
        assert_eq!(*got, vec![1, 2, 1, 1]);
    }

    #[test]
    fn parse_a_numbered_chapter() {
        let src = "- [First](./second)";
        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next();

        assert_eq!(parser.summary.numbered_chapters.len(), 0);

        parser.state = State::NumberedChapters(0);
        parser.step().unwrap();

        assert_eq!(parser.summary.numbered_chapters.len(), 1);
    }
}
