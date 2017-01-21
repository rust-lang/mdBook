#![cfg(test)]

use std::path::{Path, PathBuf};

use book::MDBook;
use renderer::{Renderer, HtmlHandlebars};
use utils;

#[ignore]// FIXME failing on Windows https://ci.appveyor.com/project/azerupi/mdbook/build/1.0.145
#[test]
fn it_renders_multilanguage_book() {
    let path = PathBuf::from(".").join("src").join("tests").join("book-wonderland-multilang");

    let renderer = HtmlHandlebars::new();
    if let Err(e) = renderer.build(&path, &None) {
        panic!("{:#?}", e);
    }

    let mut proj = MDBook::new(&path);
    proj.read_config();
    proj.parse_books();

    let mut book_path: &Path = proj.translations.get("en").unwrap().config.get_dest();
    let mut chapter_path: PathBuf;
    let mut s: String;

    // Test if index.html in the project dest folder is the main book's first chapter

    chapter_path = proj.get_dest_base().join("index.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<title>Titlepage - Alice's Adventures in Wonderland</title>"));
    assert!(s.contains("<h1>Alice's Adventures in Wonderland</h1>"));
    assert!(s.contains("<base href=\"\">"));

    // Test if each translation was rendered

    chapter_path = book_path.join("tears.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<html lang=\"en\">"));
    assert!(s.contains("<h1>The Pool of Tears</h1>"));
    assert!(s.contains("<base href=\"../\">"));

    let does_it_contain = |check_str: &str, fmt_str: &str, path_parts: Vec<&str>| -> bool {
        let mut p = PathBuf::from("");
        for i in path_parts.iter() {
            p = p.join(i);
        }
        let ps = p.to_str().unwrap();
        // Handmade formatting with replace. For anything more, it'll need Handlebars.
        let text = fmt_str.replace("{}", ps);
        check_str.contains(&text)
    };

    assert!(does_it_contain(
        &s, "<li><a href=\"{}\" class=\"active\"><strong>2.</strong> The Pool of Tears</a></li>",
        vec!["en", "tears.html"]
    ));

    book_path = proj.translations.get("fr").unwrap().config.get_dest();
    chapter_path = book_path.join("tears.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<html lang=\"fr\">"));
    assert!(s.contains("<h1>La mare aux larmes</h1>"));
    assert!(s.contains("<base href=\"../\">"));

    book_path = proj.translations.get("hu").unwrap().config.get_dest();
    chapter_path = book_path.join("tears.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<html lang=\"hu\">"));
    assert!(s.contains("<h1>Könnytó</h1>"));
    assert!(s.contains("<base href=\"../\">"));

    // Test if book's asset files were copied

    assert!(proj.get_dest_base().join("images").join("Queen.jpg").exists());

    // Test if default translation indexes are displayed

    book_path = proj.translations.get("hu").unwrap().config.get_dest();
    chapter_path = book_path.join("tarka-farka.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();

    assert!(does_it_contain(&s, "<a href=\"{}\">en</a>", vec!["en", "index.html"]));
    assert!(does_it_contain(&s, "<a href=\"{}\">hu</a>", vec!["hu", "index.html"]));
    assert!(does_it_contain(&s, "<a href=\"{}\">fr</a>", vec!["fr", "index.html"]));

    // Test if translation links are found

    book_path = proj.translations.get("en").unwrap().config.get_dest();

    chapter_path = book_path.join("rabbit-hole.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();

    assert!(does_it_contain(&s, "<a href=\"{}\">en</a>", vec!["en", "rabbit-hole.html"]));
    assert!(does_it_contain(&s, "<a href=\"{}\">hu</a>", vec!["hu", "nyuszi.html"]));
    assert!(does_it_contain(&s, "<a href=\"{}\">fr</a>", vec!["fr", "terrier.html"]));

    chapter_path = book_path.join("tears.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();

    assert!(does_it_contain(&s, "<a href=\"{}\">en</a>", vec!["en", "tears.html"]));
    assert!(does_it_contain(&s, "<a href=\"{}\">fr</a>", vec!["fr", "tears.html"]));
    assert!(does_it_contain(&s, "<a href=\"{}\">hu</a>", vec!["hu", "tears.html"]));

    chapter_path = book_path.join("long-tale.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();

    assert!(does_it_contain(&s, "<a href=\"{}\">en</a>", vec!["en", "long-tale.html"]));
    assert!(does_it_contain(&s, "<a href=\"{}\">fr</a>", vec!["fr", "cocasse.html"]));
    assert!(does_it_contain(&s, "<a href=\"{}\">hu</a>", vec!["hu", "tarka-farka.html"]));

    // Test if print.html is produced for each translations

    book_path = proj.translations.get("en").unwrap().config.get_dest();
    chapter_path = book_path.join("print.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<h1>The Pool of Tears</h1>"));

    book_path = proj.translations.get("fr").unwrap().config.get_dest();
    chapter_path = book_path.join("print.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<h1>La mare aux larmes</h1>"));

    book_path = proj.translations.get("hu").unwrap().config.get_dest();
    chapter_path = book_path.join("print.html");
    s = utils::fs::file_to_string(&chapter_path).unwrap();
    assert!(s.contains("<h1>Könnytó</h1>"));
}

#[test]
fn it_recognizes_first_translation_as_main() {
    let path = PathBuf::from(".").join("src").join("tests").join("book-noneng-main");

    let mut proj = MDBook::new(&path);
    proj.read_config();

    let t = proj.translations.get("hu").unwrap();
    assert!(t.config.is_main_book);
}
