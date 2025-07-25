use anyhow::{Context, Result};
use log::debug;
use mdbook_core::book::{Book, BookItem, Chapter};
use mdbook_core::config::BuildConfig;
use mdbook_core::utils::bracket_escape;
use mdbook_summary::{Link, Summary, SummaryItem, parse_summary};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

/// Load a book into memory from its `src/` directory.
pub(crate) fn load_book<P: AsRef<Path>>(src_dir: P, cfg: &BuildConfig) -> Result<Book> {
    let src_dir = src_dir.as_ref();
    let summary_md = src_dir.join("SUMMARY.md");

    let mut summary_content = String::new();
    File::open(&summary_md)
        .with_context(|| format!("Couldn't open SUMMARY.md in {src_dir:?} directory"))?
        .read_to_string(&mut summary_content)?;

    let summary = parse_summary(&summary_content)
        .with_context(|| format!("Summary parsing failed for file={summary_md:?}"))?;

    if cfg.create_missing {
        create_missing(src_dir, &summary).with_context(|| "Unable to create missing chapters")?;
    }

    load_book_from_disk(&summary, src_dir)
}

fn create_missing(src_dir: &Path, summary: &Summary) -> Result<()> {
    let mut items: Vec<_> = summary
        .prefix_chapters
        .iter()
        .chain(summary.numbered_chapters.iter())
        .chain(summary.suffix_chapters.iter())
        .collect();

    while let Some(next) = items.pop() {
        if let SummaryItem::Link(ref link) = *next {
            if let Some(ref location) = link.location {
                let filename = src_dir.join(location);
                if !filename.exists() {
                    if let Some(parent) = filename.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }
                    debug!("Creating missing file {}", filename.display());

                    let mut f = File::create(&filename).with_context(|| {
                        format!("Unable to create missing file: {}", filename.display())
                    })?;
                    writeln!(f, "# {}", bracket_escape(&link.name))?;
                }
            }

            items.extend(&link.nested_items);
        }
    }

    Ok(())
}

/// Use the provided `Summary` to load a `Book` from disk.
///
/// You need to pass in the book's source directory because all the links in
/// `SUMMARY.md` give the chapter locations relative to it.
pub(crate) fn load_book_from_disk<P: AsRef<Path>>(summary: &Summary, src_dir: P) -> Result<Book> {
    debug!("Loading the book from disk");
    let src_dir = src_dir.as_ref();

    let prefix = summary.prefix_chapters.iter();
    let numbered = summary.numbered_chapters.iter();
    let suffix = summary.suffix_chapters.iter();

    let summary_items = prefix.chain(numbered).chain(suffix);

    let mut chapters = Vec::new();

    for summary_item in summary_items {
        let chapter = load_summary_item(summary_item, src_dir, Vec::new())?;
        chapters.push(chapter);
    }

    Ok(Book::new_with_items(chapters))
}

fn load_summary_item<P: AsRef<Path> + Clone>(
    item: &SummaryItem,
    src_dir: P,
    parent_names: Vec<String>,
) -> Result<BookItem> {
    match item {
        SummaryItem::Separator => Ok(BookItem::Separator),
        SummaryItem::Link(link) => load_chapter(link, src_dir, parent_names).map(BookItem::Chapter),
        SummaryItem::PartTitle(title) => Ok(BookItem::PartTitle(title.clone())),
    }
}

fn load_chapter<P: AsRef<Path>>(
    link: &Link,
    src_dir: P,
    parent_names: Vec<String>,
) -> Result<Chapter> {
    let src_dir = src_dir.as_ref();

    let mut ch = if let Some(ref link_location) = link.location {
        debug!("Loading {} ({})", link.name, link_location.display());

        let location = if link_location.is_absolute() {
            link_location.clone()
        } else {
            src_dir.join(link_location)
        };

        let mut f = File::open(&location)
            .with_context(|| format!("Chapter file not found, {}", link_location.display()))?;

        let mut content = String::new();
        f.read_to_string(&mut content).with_context(|| {
            format!("Unable to read \"{}\" ({})", link.name, location.display())
        })?;

        if content.as_bytes().starts_with(b"\xef\xbb\xbf") {
            content.replace_range(..3, "");
        }

        let stripped = location
            .strip_prefix(src_dir)
            .expect("Chapters are always inside a book");

        Chapter::new(&link.name, content, stripped, parent_names.clone())
    } else {
        Chapter::new_draft(&link.name, parent_names.clone())
    };

    let mut sub_item_parents = parent_names;

    ch.number = link.number.clone();

    sub_item_parents.push(link.name.clone());
    let sub_items = link
        .nested_items
        .iter()
        .map(|i| load_summary_item(i, src_dir, sub_item_parents.clone()))
        .collect::<Result<Vec<_>>>()?;

    ch.sub_items = sub_items;

    Ok(ch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_core::book::SectionNumber;
    use std::path::PathBuf;
    use tempfile::{Builder as TempFileBuilder, TempDir};

    const DUMMY_SRC: &str = "
# Dummy Chapter

this is some dummy text.

And here is some \
                                     more text.
";

    /// Create a dummy `Link` in a temporary directory.
    fn dummy_link() -> (Link, TempDir) {
        let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();

        let chapter_path = temp.path().join("chapter_1.md");
        File::create(&chapter_path)
            .unwrap()
            .write_all(DUMMY_SRC.as_bytes())
            .unwrap();

        let link = Link::new("Chapter 1", chapter_path);

        (link, temp)
    }

    /// Create a nested `Link` written to a temporary directory.
    fn nested_links() -> (Link, TempDir) {
        let (mut root, temp_dir) = dummy_link();

        let second_path = temp_dir.path().join("second.md");

        File::create(&second_path)
            .unwrap()
            .write_all(b"Hello World!")
            .unwrap();

        let mut second = Link::new("Nested Chapter 1", &second_path);
        second.number = Some(SectionNumber(vec![1, 2]));

        root.nested_items.push(second.clone().into());
        root.nested_items.push(SummaryItem::Separator);
        root.nested_items.push(second.into());

        (root, temp_dir)
    }

    #[test]
    fn load_a_single_chapter_from_disk() {
        let (link, temp_dir) = dummy_link();
        let should_be = Chapter::new(
            "Chapter 1",
            DUMMY_SRC.to_string(),
            "chapter_1.md",
            Vec::new(),
        );

        let got = load_chapter(&link, temp_dir.path(), Vec::new()).unwrap();
        assert_eq!(got, should_be);
    }

    #[test]
    fn load_a_single_chapter_with_utf8_bom_from_disk() {
        let temp_dir = TempFileBuilder::new().prefix("book").tempdir().unwrap();

        let chapter_path = temp_dir.path().join("chapter_1.md");
        File::create(&chapter_path)
            .unwrap()
            .write_all(("\u{feff}".to_owned() + DUMMY_SRC).as_bytes())
            .unwrap();

        let link = Link::new("Chapter 1", chapter_path);

        let should_be = Chapter::new(
            "Chapter 1",
            DUMMY_SRC.to_string(),
            "chapter_1.md",
            Vec::new(),
        );

        let got = load_chapter(&link, temp_dir.path(), Vec::new()).unwrap();
        assert_eq!(got, should_be);
    }

    #[test]
    fn cant_load_a_nonexistent_chapter() {
        let link = Link::new("Chapter 1", "/foo/bar/baz.md");

        let got = load_chapter(&link, "", Vec::new());
        assert!(got.is_err());
    }

    #[test]
    fn load_recursive_link_with_separators() {
        let (root, temp) = nested_links();

        let nested = Chapter {
            name: String::from("Nested Chapter 1"),
            content: String::from("Hello World!"),
            number: Some(SectionNumber(vec![1, 2])),
            path: Some(PathBuf::from("second.md")),
            source_path: Some(PathBuf::from("second.md")),
            parent_names: vec![String::from("Chapter 1")],
            sub_items: Vec::new(),
        };
        let should_be = BookItem::Chapter(Chapter {
            name: String::from("Chapter 1"),
            content: String::from(DUMMY_SRC),
            number: None,
            path: Some(PathBuf::from("chapter_1.md")),
            source_path: Some(PathBuf::from("chapter_1.md")),
            parent_names: Vec::new(),
            sub_items: vec![
                BookItem::Chapter(nested.clone()),
                BookItem::Separator,
                BookItem::Chapter(nested),
            ],
        });

        let got = load_summary_item(&SummaryItem::Link(root), temp.path(), Vec::new()).unwrap();
        assert_eq!(got, should_be);
    }

    #[test]
    fn load_a_book_with_a_single_chapter() {
        let (link, temp) = dummy_link();
        let summary = Summary {
            numbered_chapters: vec![SummaryItem::Link(link)],
            ..Default::default()
        };
        let sections = vec![BookItem::Chapter(Chapter {
            name: String::from("Chapter 1"),
            content: String::from(DUMMY_SRC),
            path: Some(PathBuf::from("chapter_1.md")),
            source_path: Some(PathBuf::from("chapter_1.md")),
            ..Default::default()
        })];
        let should_be = Book::new_with_items(sections);

        let got = load_book_from_disk(&summary, temp.path()).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn cant_load_chapters_with_an_empty_path() {
        let (_, temp) = dummy_link();
        let summary = Summary {
            numbered_chapters: vec![SummaryItem::Link(Link {
                name: String::from("Empty"),
                location: Some(PathBuf::from("")),
                ..Default::default()
            })],

            ..Default::default()
        };

        let got = load_book_from_disk(&summary, temp.path());
        assert!(got.is_err());
    }

    #[test]
    fn cant_load_chapters_when_the_link_is_a_directory() {
        let (_, temp) = dummy_link();
        let dir = temp.path().join("nested");
        fs::create_dir(&dir).unwrap();

        let summary = Summary {
            numbered_chapters: vec![SummaryItem::Link(Link {
                name: String::from("nested"),
                location: Some(dir),
                ..Default::default()
            })],
            ..Default::default()
        };

        let got = load_book_from_disk(&summary, temp.path());
        assert!(got.is_err());
    }

    #[test]
    fn cant_open_summary_md() {
        let cfg = BuildConfig::default();
        let temp_dir = TempFileBuilder::new().prefix("book").tempdir().unwrap();

        let got = load_book(&temp_dir, &cfg);
        assert!(got.is_err());
        let error_message = got.err().unwrap().to_string();
        let expected = format!(
            r#"Couldn't open SUMMARY.md in {:?} directory"#,
            temp_dir.path()
        );
        assert_eq!(error_message, expected);
    }
}
