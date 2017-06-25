use std::error::Error;
use std::fmt::{self, Formatter, Display};
use std::ops::{Deref, DerefMut};
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
/// **Numbered Chapter:** Numbered chapters are the main content of the book, they
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

/// A stateful parser for parsing a `SUMMARY.md` file.
///
/// # Grammar
/// 
/// The `SUMMARY.md` file has a grammar which looks something like this:
///
/// ```text
/// summary           ::= title prefix_chapters numbered_chapters suffix_chapters
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
}

impl<'a> SummaryParser<'a> 
{
    fn new(text: &str) -> SummaryParser {
        let pulldown_parser = pulldown_cmark::Parser::new(text);
        let intermediate_summary = Summary::default();

        SummaryParser {
            stream: pulldown_parser,
            summary: intermediate_summary,
        }
    }

    fn parse(mut self) -> Result<Summary, Box<Error>> {
        self.summary.title = self.parse_title();

        Ok(self.summary)        
    }

    fn parse_title(&mut self) -> Option<String> {
        if let Some(Event::Start(Tag::Header(1))) = self.stream.next() {
            debug!("[*] Found a h1 in the SUMMARY");
            
            let mut tags = Vec::new();

            loop {
                let next_event = self.stream.next();
                match next_event {
                    Some(Event::End(Tag::Header(1))) => break,
                    Some(other) => tags.push(other),
                    None => {
                        // If we ever get here then changes are pulldown_cmark 
                        // is seriously broken. It means there's an opening 
                        // <h1> tag but not a closing one. It also means 
                        // we've consumed the entire stream of events, so
                        // chances are any parsing after this will just hit
                        // EOF and end early :(
                        warn!("[*] No closing <h1> tag in the SUMMARY.md file");
                        break;
                    }
                }
            }

            // TODO: How do we deal with headings like "# My **awesome** summary"?
            // for now, I'm just going to scan through and concatenate the 
            // Event::Text tags, skipping any styling.
            let title: String = tags.into_iter()
                .filter_map(|t| match t {
                    Event::Text(text) => Some(text),
                    _ => None,
                })
                .collect();

            Some(title)
        } else {
            None
        }
    }
}

/// A section number like "1.2.3", basically just a newtype'd `Vec<u32>`.
#[derive(Debug, PartialEq, Clone, Default)]
struct SectionNumber(Vec<u32>);

impl Display for SectionNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let dotted_number: String = self.0.iter().map(|i| format!("{}", i))
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
}