use crate::get_book_dir;
use anyhow::Context;
use clap::{arg, App, ArgMatches};
use lazy_static::lazy_static;
use mdbook::book::Chapter;
use mdbook::{BookItem, Config, MDBook};
use polib::catalog::Catalog;
use polib::message::Message;
use regex::Regex;
use std::path::Path;

// Create clap subcommand arguments
pub fn make_subcommand<'help>() -> App<'help> {
    App::new("xgettext")
        .about("Extract translatable strings from all chapters")
        .arg(arg!(-o --output [FILE]
                 "Write output to the specified file. Defaults to `messages.pot`."
        ))
        .arg(arg!([dir]
            "Root directory for the book{n}\
            (Defaults to the Current Directory when omitted)"
        ))
}

/// Extract paragraphs from text.
///
/// Paragraphs are separated by at least two newlines. Returns an
/// iterator over line numbers (starting from 1) and paragraphs.
pub fn extract_paragraphs(text: &str) -> impl Iterator<Item = (usize, &str)> {
    // TODO: This could be make more sophisticated by parsing the
    // Markdown and stripping off the markup characters.
    //
    // As an example, a header like "## My heading" could become just
    // "My heading" in the `.pot` file. Similarly, paragraphs could be
    // unfolded and list items could be translated one-by-one.
    lazy_static! {
        static ref PARAGRAPH_SEPARATOR: Regex = Regex::new(r"\n\n+").unwrap();
    }

    // Skip over leading empty lines.
    let trimmed = text.trim_start_matches('\n');
    let mut matches = PARAGRAPH_SEPARATOR.find_iter(trimmed);
    let mut lineno = 1 + text.len() - trimmed.len();
    let mut last = 0;

    std::iter::from_fn(move || match matches.next() {
        Some(m) => {
            let result = (lineno, &trimmed[last..m.start()]);
            lineno += trimmed[last..m.end()].lines().count();
            last = m.end();
            Some(result)
        }
        None => {
            if last < trimmed.len() {
                let result = (lineno, trimmed[last..].trim_end_matches('\n'));
                last = trimmed.len();
                Some(result)
            } else {
                None
            }
        }
    })
}

/// Split `content` into paragraphs and add them all to `catalog.`
fn add_messages<P: AsRef<Path>>(
    config: &Config,
    catalog: &mut Catalog,
    content: &str,
    reference: P,
) {
    let path = config.book.src.join(reference.as_ref());
    for (lineno, paragraph) in extract_paragraphs(content) {
        let source = format!("{}:{}", &path.display(), lineno);
        let sources = match catalog.find_message(paragraph) {
            Some(msg) => format!("{}\n{}", msg.source, source),
            None => source,
        };
        let message = Message::new_singular("", &sources, "", "", paragraph, "");
        // Carefully update the existing message or add a
        // new one. It's an error to create a catalog
        // duplicate msgids.
        match catalog.find_message_index(paragraph) {
            Some(&idx) => catalog.update_message_by_index(idx, message).unwrap(),
            None => catalog.add_message(message),
        }
    }
}

// Xgettext command implementation
pub fn execute(args: &ArgMatches) -> mdbook::errors::Result<()> {
    let book_dir = get_book_dir(args);
    let book = MDBook::load(&book_dir)?;

    let mut catalog = Catalog::new();
    catalog.metadata.content_type = String::from("text/plain; charset=UTF-8");

    let summary_path = book_dir.join(&book.config.book.src).join("SUMMARY.md");
    let summary = std::fs::read_to_string(&summary_path)?;
    add_messages(&book.config, &mut catalog, &summary, "SUMMARY.md");

    for item in book.iter() {
        if let BookItem::Chapter(Chapter {
            content,
            path: Some(path),
            ..
        }) = item
        {
            add_messages(&book.config, &mut catalog, content, path);
        }
    }

    let output_path = Path::new(args.value_of("output").unwrap_or("messages.pot"));
    polib::po_file::write(&catalog, output_path)
        .with_context(|| format!("Could not write {:?}", output_path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_iter_eq {
        ($left_iter:expr, $right:expr) => {
            assert_eq!($left_iter.collect::<Vec<_>>(), $right)
        };
    }

    #[test]
    fn test_extract_paragraphs_empty() {
        assert_iter_eq!(extract_paragraphs(""), vec![]);
    }

    #[test]
    fn test_extract_paragraphs_single_line() {
        assert_iter_eq!(
            extract_paragraphs("This is a paragraph."),
            vec![(1, "This is a paragraph.")]
        );
    }

    #[test]
    fn test_extract_paragraphs_simple() {
        assert_iter_eq!(
            extract_paragraphs("This is\na paragraph.\n\nNext paragraph."),
            vec![(1, "This is\na paragraph."), (4, "Next paragraph.")]
        );
    }

    #[test]
    fn test_extract_paragraphs_leading_newlines() {
        assert_iter_eq!(
            extract_paragraphs("\n\n\nThis is\na paragraph."),
            vec![(4, "This is\na paragraph.")]
        );
    }
}
