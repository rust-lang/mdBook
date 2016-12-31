#[cfg(test)]

use std::path::{Path, PathBuf};

use book::MDBook;
use renderer::{Renderer, HtmlHandlebars};
use utils;

#[test]
fn it_renders_html_from_minimal_book() {
    let path = PathBuf::from(".").join("src").join("tests").join("book-minimal");

    let renderer = HtmlHandlebars::new();
    if let Err(e) = renderer.build(&path) {
        println!("{:#?}", e);
    }

    let mut proj = MDBook::new(&path);
    proj.read_config();
    proj.parse_books();

    let book_path: &Path = proj.translations.get("en").unwrap().config.get_dest();
    let mut chapter_path: PathBuf = PathBuf::from("".to_string());
    let mut s: String = String::new();

    // Test if "Library of Babel" was rendered

    chapter_path = book_path.join("fictions").join("babel").with_extension("html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("The Library of Babel"));

    // Test if first chapter "Introduction" was rendered as index.html

    chapter_path = book_path.join("index.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<h1>Introduction</h1>"));

    // Test if next link from "Introduction" is "Fictions"

    chapter_path = book_path.join("index.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<a href=\"fictions.html\" class=\"nav-chapters next\""));

    // Test if previous link from "Fictions" is index.html

    chapter_path = book_path.join("fictions.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<a href=\"index.html\" class=\"nav-chapters previous\""));

}

#[test]
fn it_copies_local_assets_when_found() {
    let path = PathBuf::from(".").join("src").join("tests").join("book-minimal-with-assets");

    let renderer = HtmlHandlebars::new();
    if let Err(e) = renderer.build(&path) {
        println!("{:#?}", e);
    }

    let mut proj = MDBook::new(&path);
    proj.read_config();
    proj.parse_books();


    let book_path: &Path = proj.translations.get("en").unwrap().config.get_dest();
    let mut chapter_path: PathBuf = PathBuf::from("".to_string());
    let mut s: String = String::new();

    // Test if "Library of Babel" was rendered

    chapter_path = book_path.join("fictions").join("babel").with_extension("html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("The Library of Babel"));

    assert_eq!(book_path.join("css").join("book.css").exists(), true);
    assert_eq!(book_path.join("css").join("highlight.css").exists(), false);
}

