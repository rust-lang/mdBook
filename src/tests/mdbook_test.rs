#[cfg(test)]

extern crate toml;

use std::process::exit;
use std::path::PathBuf;

use serde_json;

use utils;
use book::MDBook;
use book::book::Book;
use book::bookconfig::BookConfig;
use book::bookconfig::Author;
use book::chapter::Chapter;
use book::toc::{TocItem, TocContent};

#[test]
fn it_parses_simple_json_config() {
    let text = r#"
{
    "title": "mdBook Documentation",
    "description": "Create books from markdown files.",
    "author": "Mathieu David"
}"#;

    let mut result = MDBook::new(&PathBuf::from("."));

    let b = utils::json_str_to_btreemap(text).unwrap();
    result.parse_from_btreemap(&b);

    let mut expected = MDBook::new(&PathBuf::from("."));

    {
        let mut book = Book::new(&PathBuf::from(result.get_project_root()));
        book.config.title =  "mdBook Documentation".to_string();
        book.config.description =  Some("Create books from markdown files.".to_string());
        book.config.authors = vec![Author::new("Mathieu David")];

        expected.translations.insert("en".to_string(), book);
    }

    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn it_parses_toml_author_without_array() {
    let text = r#"
title = "mdBook Documentation"
description = "Create books from markdown files."
author = "Mathieu David"
"#;

    let mut result = MDBook::new(&PathBuf::from("."));

    let b = utils::toml_str_to_btreemap(text).unwrap();
    result.parse_from_btreemap(&b);

    let mut expected = MDBook::new(&PathBuf::from("."));

    {
        let mut book = Book::new(&PathBuf::from(result.get_project_root()));
        book.config.title =  "mdBook Documentation".to_string();
        book.config.description =  Some("Create books from markdown files.".to_string());
        book.config.authors = vec![Author::new("Mathieu David")];

        expected.translations.insert("en".to_string(), book);
    }

    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn it_parses_simple_toml_config() {
    let text = r#"
title = "mdBook Documentation"
description = "Create books from markdown files."

[[authors]]
name = "Mathieu David"
"#;

    let mut result = MDBook::new(&PathBuf::from("."));

    let b = utils::toml_str_to_btreemap(text).unwrap();
    result.parse_from_btreemap(&b);

    let mut expected = MDBook::new(&PathBuf::from("."));

    {
        let mut book = Book::new(&PathBuf::from(result.get_project_root()));
        book.config.title =  "mdBook Documentation".to_string();
        book.config.description =  Some("Create books from markdown files.".to_string());
        book.config.authors = vec![Author::new("Mathieu David")];

        expected.translations.insert("en".to_string(), book);
    }

    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn it_parses_config_for_multiple_books() {
    let text = r#"
indent_spaces = 2

[[translations.en]]
title = "Alice's Adventures in Wonderland"
author = "Lewis Carroll"

[[translations.hu]]
title = "Alice Csodaországban"

[[translations.hu.authors]]
name = "Lewis Carroll"

[[translations.hu.translators]]
name = "Kosztolányi Dezső"
"#;

    let mut result = MDBook::new(&PathBuf::from("."));

    let mut parser = toml::Parser::new(&text);

    let config = match parser.parse() {
        Some(x) => {x},
        None => {
            error!("[*]: Toml parse errors in book.toml: {:?}", parser.errors);
            exit(2);
        }
    };

    result.parse_from_btreemap(&config);

    let mut expected = MDBook::new(&PathBuf::from("."));
    expected.indent_spaces = 2;

    {
        let mut conf = BookConfig::new(&PathBuf::from("."));
        conf.title = "Alice's Adventures in Wonderland".to_string();
        conf.authors = vec![Author::new("Lewis Carroll")];
        conf.src = expected.get_project_root().join("src").join("en");
        conf.dest = expected.get_project_root().join("book").join("en");
        conf.is_multilang = true;
        conf.is_main_book = true;

        let mut book = Book::default();
        book.config = conf;

        expected.translations.insert("en".to_string(), book);
    }

    {
        let mut conf = BookConfig::new(&PathBuf::from("."));
        conf.title = "Alice Csodaországban".to_string();
        conf.authors = vec![Author::new("Lewis Carroll")];
        conf.translators = Some(vec![Author::new("Kosztolányi Dezső")]);
        conf.src = expected.get_project_root().join("src").join("hu");
        conf.dest = expected.get_project_root().join("book").join("hu");
        conf.is_multilang = true;
        conf.is_main_book = false;

        let mut book = Book::default();
        book.config = conf;

        expected.translations.insert("hu".to_string(), book);
    }

    // Hashmaps are unordered. They don't always print their keys in the same order.

    assert_eq!(
        format!("{:#?} {:#?} {:#?}", result.indent_spaces, result.translations.get("en").unwrap(), result.translations.get("hu").unwrap()),
        format!("{:#?} {:#?} {:#?}", expected.indent_spaces, expected.translations.get("en").unwrap(), expected.translations.get("hu").unwrap())
    );
}

#[test]
fn it_parses_config_for_a_single_book() {
    let text = r#"
indent_spaces = 2
title = "Alice Csodaországban"
language = { name = "Hungarian", code = "hu" }

[[authors]]
name = "Lewis Carroll"

[[translators]]
name = "Kosztolányi Dezső"
"#;

    let mut result = MDBook::new(&PathBuf::from("."));

    let mut parser = toml::Parser::new(&text);

    let config = match parser.parse() {
        Some(x) => {x},
        None => {
            error!("[*]: Toml parse errors in book.toml: {:?}", parser.errors);
            exit(2);
        }
    };

    result.parse_from_btreemap(&config);

    let mut expected = MDBook::new(&PathBuf::from("."));
    expected.indent_spaces = 2;

    {
        let mut book = Book::new(&PathBuf::from(result.get_project_root()));
        book.config.title = "Alice Csodaországban".to_string();
        book.config.authors = vec![Author::new("Lewis Carroll")];
        book.config.translators = Some(vec![Author::new("Kosztolányi Dezső")]);
        book.config.language.name = "Hungarian".to_string();
        book.config.language.code = "hu".to_string();

        expected.translations.insert("hu".to_string(), book);
    }

    assert_eq!(
        format!("{:?} {:?}", result.indent_spaces, result.translations.get("hu").unwrap()),
        format!("{:?} {:?}", expected.indent_spaces, expected.translations.get("hu").unwrap())
    );
}

#[test]
fn it_parses_toc_and_chapters_in_minimal_book() {
    let path = PathBuf::from(".").join("src").join("tests").join("book-minimal");
    let mut result = MDBook::new(&path);

    result.read_config();
    result.parse_books();

    let mut babel = Chapter::default();
    if let TocItem::Numbered(ref fictions) = result.translations.get("en").unwrap().toc[1] {
        if let TocItem::Numbered(ref ch) = fictions.clone().sub_items.unwrap()[1] {
            babel = ch.chapter.clone();
        }
    }

    assert_eq!(format!("{:?}", babel.translators.unwrap()), format!("{:?}", vec![Author::new("James E. Irby")]));
}
