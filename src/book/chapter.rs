use book::metadata::BookMetadata;

use std::path;

/// The Chapter struct holds the title of the chapter as written in the SUMMARY.md file,
/// the location of the markdown file containing the content and eventually sub-chapters
pub struct Chapter {
    title: String,
    file: path::PathBuf,

    sub_chapters: Vec<Chapter>,
}


impl Chapter {
    /// Creates a new chapter with the given title and source file and no sub-chapters
    pub fn new(title: &str, file: &path::Path) -> Self {
        Chapter {
            title: title.to_owned(),
            file: file.to_owned(),

            sub_chapters: Vec::new(),
        }
    }

    /// This function takes a slice `&[x,y,z]` and returns the corresponding sub-chapter if it exists.
    ///
    /// For example: `chapter.get_sub_chapter(&[1,3])` will return the third sub-chapter of the first sub-chapter.
    pub fn get_sub_chapter(&self, section: &[usize]) -> Option<&Chapter> {
        match section.len() {
            0 => None,
            1 => self.sub_chapters.get(section[0]),
            _ => {
                // The lengt of the slice is more than one, this means that we want a sub-chapter of a sub-chapter
                // We call `get_sub_chapter` recursively until we are deep enough and return the asked sub-chapter
                self.sub_chapters
                    .get(section[0])
                    .and_then(|ch| ch.get_sub_chapter(&section[1..]))
            },
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn file(&self) -> &path::Path {
        &self.file
    }
    pub fn sub_chapters(&self) -> &[Chapter] {
        &self.sub_chapters
    }
}
