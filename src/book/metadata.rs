#[derive(Debug, Clone)]
pub struct BookMetadata {
    pub title: String,
    pub description: String,

    pub language: Language,

    authors: Vec<Author>,
    translators: Vec<Author>,
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


impl BookMetadata {
    pub fn new(title: &str) -> Self {
        BookMetadata {
            title: title.to_owned(),
            description: String::new(),

            language: Language::default(),

            authors: Vec::new(),
            translators: Vec::new(),
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
