use std::error::Error;
use pulldown_cmark;

/// The parsed `SUMMARY.md`, specifying how the book should be laid out.
pub struct Summary;

pub fn parse_summary(summary: &str) -> Result<Summary, Box<Error>> {
    unimplemented!()
}