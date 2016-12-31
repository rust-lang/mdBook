use book::chapter::Chapter;

/// A Table of Contents is a `Vec<TocItem>`, where an item is an enum that
/// qualifies its content.
#[derive(Debug, Clone)]
pub enum TocItem {
    Numbered(TocContent),
    Unnumbered(TocContent),
    Unlisted(TocContent),
    Spacer,
}

/// An entry in the TOC with content. Its payload is the Chapter. This struct
/// knows the section index of the entry, or contains optional sub-entries as
/// `Vec<TocItem>`.
#[derive(Debug, Clone)]
pub struct TocContent {
    pub chapter: Chapter,
    pub sub_items: Option<Vec<TocItem>>,
    /// Section indexes of the chapter
    pub section: Option<Vec<i32>>,
}

impl Default for TocContent {
    fn default() -> TocContent {
        TocContent {
            chapter: Chapter::default(),
            sub_items: None,
            section: None,
        }
    }
}

impl TocContent {

    pub fn new(chapter: Chapter) -> TocContent {
        let mut toc = TocContent::default();
        toc.chapter = chapter;
        toc
    }

    pub fn new_with_section(chapter: Chapter, section: Vec<i32>) -> TocContent {
        let mut toc = TocContent::default();
        toc.chapter = chapter;
        toc.section = Some(section);
        toc
    }

    pub fn section_as_string(&self) -> String {
        if let Some(ref sec) = self.section {
            let a = sec.iter().map(|x| x.to_string()).collect::<Vec<String>>();
            format!("{}.", a.join("."))
        } else {
            "".to_string()
        }
    }

    // TODO update

    // /// This function takes a slice `&[x,y,z]` and returns the corresponding sub-chapter if it exists.
    // ///
    // /// For example: `chapter.get_sub_chapter(&[1,3])` will return the third sub-chapter of the first sub-chapter.
    // pub fn get_sub_chapter(&self, section: &[usize]) -> Option<&Chapter> {
    //     match section.len() {
    //         0 => None,
    //         1 => self.sub_chapters.get(section[0]),
    //         _ => {
    //             // The lengt of the slice is more than one, this means that we want a sub-chapter of a sub-chapter
    //             // We call `get_sub_chapter` recursively until we are deep enough and return the asked sub-chapter
    //             self.sub_chapters
    //                 .get(section[0])
    //                 .and_then(|ch| ch.get_sub_chapter(&section[1..]))
    //         },
    //     }
    // }
}
