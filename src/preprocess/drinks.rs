use crate::errors::*;

use super::{Preprocessor, PreprocessorContext};
use crate::book::{Book, BookItem, Chapter};
use regex::{Captures, Regex};
use once_cell::sync::Lazy;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::collections::HashMap;

const SPLITTER: char = ':';

type Dict=HashMap<String, String>;

/// DRY Links - A preprocessor for using centralized links collection:
///
/// - `{{# drink}}` - Insert link from the collection
#[derive(Default)]
pub struct DrinkPreprocessor;

impl DrinkPreprocessor {
    pub(crate) const NAME: &'static str = "drinks";

    /// Create a new `DrinkPreprocessor`.
    pub fn new() -> Self {
        DrinkPreprocessor
    }

    fn replace_drinks(&self, chapter: &mut Chapter, dict: &Dict) -> Result<String, Error> {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
            r"(?x)                # insignificant whitespace mode
            \{\{\s*               # link opening parens and whitespace
            \#(drink)             # drink marker
            \s+                   # separating whitespace
            (?<drink>[A-z0-9_-]+) # drink name
            \}\}                  # link closing parens",
            ).unwrap()
        });

        static NODRINK: Lazy<String> = Lazy::new(|| {
            "deadbeef".to_string()
        });

        let res = RE.replace_all(&chapter.content, |caps: &Captures<'_>| {
            dict.get(&caps["drink"]).unwrap_or(&NODRINK)
        });
        Ok(res.to_string())
    }
}

impl Preprocessor for DrinkPreprocessor {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let path = ctx.root.join("drinks.txt");

        let drinks: Dict = {
            let reader = BufReader::new(File::open(path).expect("Cannot open drinks dictionary"));
            reader.lines().filter_map(|l| {
                l.expect("Cannot read line in drinks dictionary").split_once(SPLITTER).map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
            }).collect::<HashMap<_, _>>()
        };

        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut ch) = *section {
                ch.content = self
                    .replace_drinks(ch, &drinks)
                    .expect("Error converting drinks into links for chapter");
            }
        });

        Ok(book)
    }
}
