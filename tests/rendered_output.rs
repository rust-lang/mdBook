extern crate mdbook;
extern crate tempdir;

mod helpers;
use mdbook::MDBook;


/// Make sure you can load the dummy book and build it without panicking.
#[test]
fn build_the_dummy_book() {
    let temp = helpers::DummyBook::default().build();
    let mut md = MDBook::new(temp.path());

    md.build().unwrap();
}

#[test]
fn by_default_mdbook_generates_rendered_content_in_the_book_directory() {
    let temp = helpers::DummyBook::default().build();
    let mut md = MDBook::new(temp.path());

    assert!(!temp.path().join("book").exists());
    md.build().unwrap();

    assert!(temp.path().join("book").exists());
    assert!(temp.path().join("book").join("index.html").exists());
}

#[test]
fn make_sure_bottom_level_files_contain_links_to_chapters() {
    let temp = helpers::DummyBook::default().build();
    let mut md = MDBook::new(temp.path());
    md.build().unwrap();

    let dest = temp.path().join("book");
    let links = vec![
        "intro.html",
        "first/index.html",
        "first/nested.html",
        "second.html",
        "conclusion.html",
    ];

    let files_in_bottom_dir = vec!["index.html", "intro.html", "second.html", "conclusion.html"];

    for filename in files_in_bottom_dir {
        helpers::assert_contains_strings(dest.join(filename), &links);
    }
}

#[test]
fn check_correct_cross_links_in_nested_dir() {
    let temp = helpers::DummyBook::default().build();
    let mut md = MDBook::new(temp.path());
    md.build().unwrap();

    let first = temp.path().join("book").join("first");
    let links = vec![
        r#"<base href="../">"#,
        "intro.html",
        "first/index.html",
        "first/nested.html",
        "second.html",
        "conclusion.html",
    ];

    let files_in_nested_dir = vec!["index.html", "nested.html"];

    for filename in files_in_nested_dir {
        helpers::assert_contains_strings(first.join(filename), &links);
    }
}

#[test]
fn rendered_code_has_playpen_stuff() {
    let temp = helpers::DummyBook::default().build();
    let mut md = MDBook::new(temp.path());
    md.build().unwrap();

    let nested = temp.path().join("book/first/nested.html");
    let playpen_class = vec![r#"class="playpen""#];

    helpers::assert_contains_strings(nested, &playpen_class);

    let book_js = temp.path().join("book/book.js");
    helpers::assert_contains_strings(book_js, &[".playpen"]);
}

#[test]
fn chapter_content_appears_in_rendered_document() {
    let content = vec![
        ("index.html", "Here's some interesting text"),
        ("second.html", "Second Chapter"),
        ("first/nested.html", "testable code"),
        ("first/index.html", "more text"),
        ("conclusion.html", "Conclusion"),
    ];

    let temp = helpers::DummyBook::default().build();
    let mut md = MDBook::new(temp.path());
    md.build().unwrap();

    let destination = temp.path().join("book");

    for (filename, text) in content {
        let path = destination.join(filename);
        helpers::assert_contains_strings(path, &[text]);
    }
}