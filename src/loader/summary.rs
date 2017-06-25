use std::error::Error;
use std::fmt::{self, Formatter, Display};
use std::ops::{Deref, DerefMut};
use pulldown_cmark;


/// The parsed `SUMMARY.md`, specifying how the book should be laid out.
pub struct Summary {
    title: Option<String>,
}

/// Parse the text from a `SUMMARY.md` file into a sort of "recipe" to be
/// used when loading a book from disk.
pub fn parse_summary(summary: &str) -> Result<Summary, Box<Error>> {
    unimplemented!()
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
}