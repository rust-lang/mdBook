use std::path::PathBuf;

/// TODO use in template: subtitle, description, publisher, number_format, section_names

#[derive(Debug, Clone)]
pub struct BookMetadata {
    /// The title of the book.
    pub title: String,

    /// TODO The subtitle, when titles are in the form of "The Immense Journey: An
    /// Imaginative Naturalist Explores the Mysteries of Man and Nature"
    pub subtitle: String,

    /// TODO A brief description or summary.
    pub description: String,

    /// TODO Publisher's info
    pub publisher: Publisher,

    pub language: Language,

    authors: Vec<Author>,
    translators: Vec<Author>,

    /// TODO Chapter numbering scheme
    number_format: NumberFormat,
    /// TODO Section names for nested Vec<Chapter> structures, such as `[
    /// "Part", "Chapter", "Section" ]`
    section_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Author {
    name: String,
    email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Language {
    name: String,
    code: String,
}

/// TODO use Publisher in template.

#[derive(Debug, Clone)]
pub struct Publisher {
    name: String,
    /// link to the sublisher's site
    url: String,
    /// path to publisher's logo image
    logo_src: PathBuf,
}

impl Publisher {
    pub fn default() -> Publisher {
        Publisher {
            name: "".to_string(),
            url: "".to_string(),
            logo_src: PathBuf::new(),
        }
    }
}

/// TODO use NumberFormat when rendering chapter titles.

#[derive(Debug, Clone)]
pub enum NumberFormat {
    /// 19
    Arabic,
    /// XIX
    Roman,
    /// Nineteen
    Word,
}

impl BookMetadata {
    pub fn new(title: &str) -> Self {
        BookMetadata {
            title: title.to_owned(),
            description: String::new(),

            language: Language::default(),

            authors: Vec::new(),
            translators: Vec::new(),

            // TODO placeholder values for now
            subtitle: "".to_string(),
            publisher: Publisher::default(),
            number_format: NumberFormat::Arabic,
            section_names: vec![],
        }
    }

    pub fn set_description(&mut self, description: &str) -> &mut Self {
        self.description = description.to_owned();
        self
    }

    pub fn add_author(&mut self, author: Author) -> &mut Self {
        self.authors.push(author);
        self
    }
}

impl Author {
    pub fn new(name: &str) -> Self {
        Author {
            name: name.to_owned(),
            email: None,
        }
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = Some(email.to_owned());
        self
    }
}


impl Default for Language {
    fn default() -> Self {
        Language {
            name: String::from("English"),
            code: String::from("en"),
        }
    }
}
