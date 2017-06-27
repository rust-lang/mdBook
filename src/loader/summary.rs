use std::error::Error;
use std::fmt::{self, Formatter, Display};
use std::io::{Error as IoError, ErrorKind};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use pulldown_cmark::{self, Event, Tag};


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
pub fn parse_summary(summary: &str) -> Result<Summary, Box<Error>> {
    let parser = SummaryParser::new(summary);
    parser.parse()
}

/// The parsed `SUMMARY.md`, specifying how the book should be laid out.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Summary {
    title: Option<String>,
}

/// A struct representing an entry in the `SUMMARY.md`, possibly with nested
/// entries.
///
/// This is roughly the equivalent of `[Some section](./path/to/file.md)`.
#[derive(Debug, Clone, Default, PartialEq)]
struct Link {
    name: String,
    location: PathBuf,
    nested_items: Vec<SummaryItem>,
}

#[derive(Debug, Clone, PartialEq)]
enum SummaryItem {
    Link(Link),
    Separator,
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

/// A stateful parser for parsing a `SUMMARY.md` file.
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
    fn parse(mut self) -> Result<Summary, Box<Error>> {
        self.summary.title = self.parse_title();

        Ok(self.summary)
    }

    fn step(&mut self) -> Result<(), Box<Error>> {
        let next_event = self.stream.next().expect("TODO: error-chain");
        trace!("[*] Current state = {:?}, Next Event = {:?}", self.state, next_event);
    
        match self.state {
            State::Begin => self.step_start(next_event),
            other => unimplemented!()
        }
    }

    /// The very first state, we should see a `BeginParagraph` token or
    /// it's an error...
    fn step_start(&mut self, event: Event<'a>) -> Result<(), Box<Error>> {
        match event {
            Event::Start(Tag::Paragraph) => self.state = State::PrefixChapters,
            other => panic!("Unexpected tag! {:?}", other),
        }

        Ok(())
    }

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

    /// Parse a single item (`[Some Chapter Name](./path/to/chapter.md)`).
    fn parse_item(&mut self) -> Result<Link, Box<Error>> {
        let next = self.stream.next();

        if let Some(Event::Start(Tag::Link(dest, _))) = next {
            let content = collect_events!(self.stream, Tag::Link(..));

            Ok(Link {
                name: stringify_events(content),
                location: PathBuf::from(dest.to_string()),
                nested_items: Vec::new(),
            })
        } else {
            Err(Box::new(IoError::new(ErrorKind::Other, format!("Expected a link, got {:?}", next))))
        }
    }
}

/// Extracts the text from formatted markdown.
fn stringify_events<'a>(events: Vec<Event<'a>>) -> String {
    events
        .into_iter()
        .filter_map(|t| match t {
            Event::Text(text) => Some(text.into_owned()),
            _ => None,
        })
        .collect()
}

/// A section number like "1.2.3", basically just a newtype'd `Vec<u32>`.
#[derive(Debug, PartialEq, Clone, Default)]
struct SectionNumber(Vec<u32>);

impl Display for SectionNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let dotted_number: String = self.0
            .iter()
            .map(|i| format!("{}", i))
            .collect::<Vec<String>>()
            .join(".");

        write!(f, "{}", dotted_number)
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
            (vec![0], "0"),
            (vec![1, 3], "1.3"),
            (vec![1, 2, 3], "1.2.3"),
        ];

        for (input, should_be) in inputs {
            let section_number = SectionNumber(input);
            let string_repr = format!("{}", section_number);

            assert_eq!(string_repr, should_be);
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
}
