use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use nom_bibtex::*;

use super::summary::{parse_summary, Link, SectionNumber, Summary, SummaryItem};
use crate::config::BuildConfig;
use crate::errors::*;
use crate::utils::fs::get_filename_extension;

/// Load a book into memory from its `src/` directory.
pub fn load_book<P: AsRef<Path>>(
    src_dir: P,
    cfg: &BuildConfig,
    bibliography_file: PathBuf,
) -> Result<Book> {
    let src_dir = src_dir.as_ref();
    let summary_md = src_dir.join("SUMMARY.md");

    let mut summary_content = String::new();
    File::open(summary_md)
        .with_context(|| "Couldn't open SUMMARY.md")?
        .read_to_string(&mut summary_content)?;

    // We have to make summary mutable to add the bibliography
    let mut summary = parse_summary(&summary_content).with_context(|| "Summary parsing failed")?;

    // Add the bibliography to the book structure if it has been specified in the config and the file exists
    // TODO Maybe add the check for the .bib extension here
    if !bibliography_file.to_str().unwrap_or_default().is_empty()
        && src_dir.join(bibliography_file.clone()).exists()
    {
        info!("Adding a bibliography to the summary!!!");
        summary.suffix_chapters.push(SummaryItem::Bibliography(
            "Bibliography".to_owned(),
            bibliography_file,
        ));
    }

    if cfg.create_missing {
        create_missing(&src_dir, &summary).with_context(|| "Unable to create missing chapters")?;
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

    while !items.is_empty() {
        let next = items.pop().expect("already checked");

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

                    let mut f = File::create(&filename)?;
                    writeln!(f, "# {}", link.name)?;
                }
            }

            items.extend(&link.nested_items);
        }
    }

    Ok(())
}

/// A dumb tree structure representing a book.
///
/// For the moment a book is just a collection of `BookItems` which are
/// accessible by either iterating (immutably) over the book with [`iter()`], or
/// recursively applying a closure to each section to mutate the chapters, using
/// [`for_each_mut()`].
///
/// [`iter()`]: #method.iter
/// [`for_each_mut()`]: #method.for_each_mut
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Book {
    /// The sections in this book.
    pub sections: Vec<BookItem>,
    /// List of bibliographic entries: <citation-key, BibItem info>.
    pub bibliography: HashMap<String, BibItem>,
    __non_exhaustive: (),
}

impl Book {
    /// Create an empty book.
    pub fn new() -> Self {
        Default::default()
    }

    /// Get a depth-first iterator over the items in the book.
    pub fn iter(&self) -> BookItems<'_> {
        BookItems {
            items: self.sections.iter().collect(),
        }
    }

    /// Recursively apply a closure to each item in the book, allowing you to
    /// mutate them.
    ///
    /// # Note
    ///
    /// Unlike the `iter()` method, this requires a closure instead of returning
    /// an iterator. This is because using iterators can possibly allow you
    /// to have iterator invalidation errors.
    pub fn for_each_mut<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut BookItem),
    {
        for_each_mut(&mut func, &mut self.sections);
    }

    /// Append a `BookItem` to the `Book`.
    pub fn push_item<I: Into<BookItem>>(&mut self, item: I) -> &mut Self {
        self.sections.push(item.into());
        self
    }
}

pub fn for_each_mut<'a, F, I>(func: &mut F, items: I)
where
    F: FnMut(&mut BookItem),
    I: IntoIterator<Item = &'a mut BookItem>,
{
    for item in items {
        if let BookItem::Chapter(ch) = item {
            for_each_mut(func, &mut ch.sub_items);
        }

        func(item);
    }
}

/// Enum representing any type of item which can be added to a book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BookItem {
    /// A nested chapter.
    Chapter(Chapter),
    /// A section separator.
    Separator,
    /// A part title.
    PartTitle(String),
    /// A Bibliography (treated as special Chapter).
    Bibliography(Chapter, Vec<BibItem>),
}

impl From<Chapter> for BookItem {
    fn from(other: Chapter) -> BookItem {
        BookItem::Chapter(other)
    }
}

/// The representation of a "chapter", usually mapping to a single file on
/// disk however it may contain multiple sub-chapters.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    /// The chapter's name.
    pub name: String,
    /// The chapter's contents.
    pub content: String,
    /// The chapter's section number, if it has one.
    pub number: Option<SectionNumber>,
    /// Nested items.
    pub sub_items: Vec<BookItem>,
    /// The chapter's location, relative to the `SUMMARY.md` file.
    pub path: Option<PathBuf>,
    /// An ordered list of the names of each chapter above this one, in the hierarchy.
    pub parent_names: Vec<String>,
}

impl Chapter {
    /// Create a new chapter with the provided content.
    pub fn new<P: Into<PathBuf>>(
        name: &str,
        content: String,
        path: P,
        parent_names: Vec<String>,
    ) -> Chapter {
        Chapter {
            name: name.to_string(),
            content,
            path: Some(path.into()),
            parent_names,
            ..Default::default()
        }
    }

    /// Create a new draft chapter that is not attached to a source markdown file and has
    /// thus no content.
    pub fn new_draft(name: &str, parent_names: Vec<String>) -> Self {
        Chapter {
            name: name.to_string(),
            content: String::new(),
            path: None,
            parent_names,
            ..Default::default()
        }
    }

    /// Check if the chapter is a draft chapter, meaning it has no path to a source markdown file
    pub fn is_draft_chapter(&self) -> bool {
        match self.path {
            Some(_) => false,
            None => true,
        }
    }
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

    // Check if the last chapter was marked as a Bibliography and if so, create the <citation-key, BibItem> map
    let last_chapter = chapters.last().unwrap();
    let mut bibliography: HashMap<String, BibItem> = HashMap::new();
    match last_chapter {
        BookItem::Bibliography(_, bib) => {
            info!("Bibliography recovered from last chapter!!!");
            for b in bib.iter() {
                bibliography.insert(b.citation_key.to_owned(), b.to_owned());
            }
        }
        _ => (),
    }

    Ok(Book {
        sections: chapters,
        bibliography: bibliography,
        __non_exhaustive: (),
    })
}

fn load_summary_item<P: AsRef<Path> + Clone>(
    item: &SummaryItem,
    src_dir: P,
    parent_names: Vec<String>,
) -> Result<BookItem> {
    match item {
        SummaryItem::Separator => Ok(BookItem::Separator),
        SummaryItem::Link(ref link) => {
            load_chapter(link, src_dir, parent_names).map(BookItem::Chapter)
        }
        SummaryItem::PartTitle(title) => Ok(BookItem::PartTitle(title.clone())),
        SummaryItem::Bibliography(title, file) => {
            let bibliography = load_bibliography(src_dir.as_ref().join(&file));
            Ok(BookItem::Bibliography(
                Chapter::new(
                    title,
                    String::from("# Bibliography\n\n"),
                    &file,
                    parent_names.clone(),
                ),
                bibliography.unwrap(),
            ))
        }
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

        let stripped = location
            .strip_prefix(&src_dir)
            .expect("Chapters are always inside a book");

        Chapter::new(&link.name, content, stripped, parent_names.clone())
    } else {
        Chapter::new_draft(&link.name, parent_names.clone())
    };

    let mut sub_item_parents = parent_names.clone();

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

/// Bibliography item representation.
/// TODO: Complete with more fields when necessary
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BibItem {
    /// The citation key.
    pub citation_key: String,
    /// The article's title.
    pub title: String,
    /// The article's author/s.
    pub authors: Vec<String>,
    /// Pub date.
    pub pub_date: String,
    /// Summary/Abstract.
    pub summary: String,
}

impl BibItem {
    /// Create a new bib item with the provided content.
    pub fn new(
        citation_key: &str,
        title: String,
        authors: Vec<String>,
        pub_date: String,
        summary: String,
    ) -> BibItem {
        BibItem {
            citation_key: citation_key.to_string(),
            title: title,
            authors: authors,
            pub_date: pub_date,
            summary: summary,
        }
    }
}

/// Load bibliography from file.
/// TODO: This can return directly a map <citation-key, BibItem> to avoid further conversions.
pub(crate) fn load_bibliography<P: AsRef<Path>>(biblio_file: P) -> Result<Vec<BibItem>> {
    info!("Loading bibliography from {:?}...", biblio_file.as_ref());

    let biblio_file_ext = get_filename_extension(biblio_file.as_ref());
    if biblio_file_ext.unwrap_or_default().to_lowercase() != "bib" {
        warn!(
            "Only bib-based bibliography is supported for now! Yours: {:?}",
            biblio_file.as_ref()
        );
        let out: Vec<BibItem> = Vec::new();
        return Ok(out);
    }

    let bibtex_content = fs::read_to_string(biblio_file)?.to_string();

    let bibtex = Bibtex::parse(&bibtex_content).unwrap();

    let biblio = bibtex.bibliographies();
    info!("{} bibliography items read", biblio.len());

    let bibliography: Vec<BibItem> = biblio
        .into_iter()
        .map(|bib| {
            let tm: HashMap<String, String> = bib.tags().into_iter().map(|t| t.clone()).collect();
            let mut authors_str = tm.get("author").unwrap().to_string();
            authors_str.retain(|c| c != '\n');
            let authors: Vec<String> = authors_str
                .split("and")
                .map(|a| a.trim().to_string())
                .collect();
            BibItem {
                citation_key: bib.citation_key().to_string(),
                title: tm
                    .get("title")
                    .unwrap_or(&"Not Found".to_owned())
                    .to_string(),
                authors: authors,
                pub_date: [
                    tm.get("month").unwrap().to_string(),
                    tm.get("year").unwrap().to_string(),
                ]
                .join(" "),
                summary: tm
                    .get("abstract")
                    .unwrap_or(&"Not Found".to_owned())
                    .to_string(),
            }
        })
        .collect();
    debug!("Bibiography content:\n{:?}", bibliography);

    Ok(bibliography)
}

/// A depth-first iterator over the items in a book.
///
/// # Note
///
/// This struct shouldn't be created directly, instead prefer the
/// [`Book::iter()`] method.
///
/// [`Book::iter()`]: struct.Book.html#method.iter
pub struct BookItems<'a> {
    items: VecDeque<&'a BookItem>,
}

impl<'a> Iterator for BookItems<'a> {
    type Item = &'a BookItem;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.pop_front();

        if let Some(&BookItem::Chapter(ref ch)) = item {
            // if we wanted a breadth-first iterator we'd `extend()` here
            for sub_item in ch.sub_items.iter().rev() {
                self.items.push_front(sub_item);
            }
        }

        item
    }
}

impl Display for Chapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref section_number) = self.number {
            write!(f, "{} ", section_number)?;
        }

        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{Builder as TempFileBuilder, TempDir};

    const DUMMY_SRC: &str = "
# Dummy Chapter

this is some dummy text.

And here is some \
                                     more text.
";

    const DUMMY_BIB_SRC: &str = "
@misc {fps,
    title = \"This is a bib entry!\",
    author = \"Francisco Perez-Sorrosal\",
    month = \"oct\",
    year = \"2020\"
}
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
        root.nested_items.push(second.clone().into());

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
            parent_names: vec![String::from("Chapter 1")],
            sub_items: Vec::new(),
        };
        let should_be = BookItem::Chapter(Chapter {
            name: String::from("Chapter 1"),
            content: String::from(DUMMY_SRC),
            number: None,
            path: Some(PathBuf::from("chapter_1.md")),
            parent_names: Vec::new(),
            sub_items: vec![
                BookItem::Chapter(nested.clone()),
                BookItem::Separator,
                BookItem::Chapter(nested.clone()),
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
        let should_be = Book {
            sections: vec![BookItem::Chapter(Chapter {
                name: String::from("Chapter 1"),
                content: String::from(DUMMY_SRC),
                path: Some(PathBuf::from("chapter_1.md")),
                ..Default::default()
            })],
            ..Default::default()
        };

        let got = load_book_from_disk(&summary, temp.path()).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn book_iter_iterates_over_sequential_items() {
        let book = Book {
            sections: vec![
                BookItem::Chapter(Chapter {
                    name: String::from("Chapter 1"),
                    content: String::from(DUMMY_SRC),
                    ..Default::default()
                }),
                BookItem::Separator,
            ],
            ..Default::default()
        };

        let should_be: Vec<_> = book.sections.iter().collect();

        let got: Vec<_> = book.iter().collect();

        assert_eq!(got, should_be);
    }

    #[test]
    fn iterate_over_nested_book_items() {
        let book = Book {
            sections: vec![
                BookItem::Chapter(Chapter {
                    name: String::from("Chapter 1"),
                    content: String::from(DUMMY_SRC),
                    number: None,
                    path: Some(PathBuf::from("Chapter_1/index.md")),
                    parent_names: Vec::new(),
                    sub_items: vec![
                        BookItem::Chapter(Chapter::new(
                            "Hello World",
                            String::new(),
                            "Chapter_1/hello.md",
                            Vec::new(),
                        )),
                        BookItem::Separator,
                        BookItem::Chapter(Chapter::new(
                            "Goodbye World",
                            String::new(),
                            "Chapter_1/goodbye.md",
                            Vec::new(),
                        )),
                    ],
                }),
                BookItem::Separator,
            ],
            ..Default::default()
        };

        let got: Vec<_> = book.iter().collect();

        assert_eq!(got.len(), 5);

        // checking the chapter names are in the order should be sufficient here...
        let chapter_names: Vec<String> = got
            .into_iter()
            .filter_map(|i| match *i {
                BookItem::Chapter(ref ch) => Some(ch.name.clone()),
                _ => None,
            })
            .collect();
        let should_be: Vec<_> = vec![
            String::from("Chapter 1"),
            String::from("Hello World"),
            String::from("Goodbye World"),
        ];

        assert_eq!(chapter_names, should_be);
    }

    #[test]
    fn for_each_mut_visits_all_items() {
        let mut book = Book {
            sections: vec![
                BookItem::Chapter(Chapter {
                    name: String::from("Chapter 1"),
                    content: String::from(DUMMY_SRC),
                    number: None,
                    path: Some(PathBuf::from("Chapter_1/index.md")),
                    parent_names: Vec::new(),
                    sub_items: vec![
                        BookItem::Chapter(Chapter::new(
                            "Hello World",
                            String::new(),
                            "Chapter_1/hello.md",
                            Vec::new(),
                        )),
                        BookItem::Separator,
                        BookItem::Chapter(Chapter::new(
                            "Goodbye World",
                            String::new(),
                            "Chapter_1/goodbye.md",
                            Vec::new(),
                        )),
                    ],
                }),
                BookItem::Separator,
            ],
            ..Default::default()
        };

        let num_items = book.iter().count();
        let mut visited = 0;

        book.for_each_mut(|_| visited += 1);

        assert_eq!(visited, num_items);
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
    fn load_bib_bibliography_from_file() {
        let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();

        let chapter_path = temp.path().join("biblio.bib");
        File::create(&chapter_path)
            .unwrap()
            .write_all(DUMMY_BIB_SRC.as_bytes())
            .unwrap();

        let bibliography_loaded: Vec<BibItem> = load_bibliography(chapter_path.as_path()).unwrap();
        assert_eq!(bibliography_loaded.len(), 1);
        assert_eq!(bibliography_loaded[0].citation_key, "fps".to_owned());
        // TODO: Add more asserts if required
    }

    #[test]
    fn cant_load_bib_bibliography_from_file() {
        let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();

        let chapter_path = temp.path().join("biblio.wrong_extension");
        File::create(&chapter_path)
            .unwrap()
            .write_all(DUMMY_BIB_SRC.as_bytes())
            .unwrap();

        let bibliography_loaded: Vec<BibItem> = load_bibliography(chapter_path.as_path()).unwrap();
        assert_eq!(bibliography_loaded.len(), 0);
    }

    #[test]
    fn load_a_book_with_a_bibliography() {
        let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();

        let biblio_path = temp.path().join("biblio.bib");
        File::create(&biblio_path)
            .unwrap()
            .write_all(DUMMY_BIB_SRC.as_bytes())
            .unwrap();

        let bibliography_loaded: Vec<BibItem> =
            load_bibliography(biblio_path.to_owned().as_path()).unwrap();
        let mut the_bibliography: HashMap<String, BibItem> = HashMap::new();
        for b in bibliography_loaded.iter() {
            the_bibliography.insert(b.citation_key.to_owned(), b.to_owned());
        }

        let should_be = Book {
            sections: vec![BookItem::Bibliography(
                Chapter {
                    name: String::from("Bibliography"),
                    content: String::from("# Bibliography\n\n"),
                    path: Some(PathBuf::from(temp.path().join("biblio.bib"))),
                    ..Default::default()
                },
                bibliography_loaded,
            )],
            bibliography: the_bibliography,
            ..Default::default()
        };

        let summary = Summary {
            suffix_chapters: vec![SummaryItem::Bibliography(
                "Bibliography".to_owned(),
                biblio_path.to_owned(),
            )],
            ..Default::default()
        };

        let got = load_book_from_disk(&summary, temp.path()).unwrap();

        assert_eq!(got, should_be);
    }
}
