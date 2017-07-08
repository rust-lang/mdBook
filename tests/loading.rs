//! Integration tests for loading a book into memory

#[macro_use]
extern crate pretty_assertions;
extern crate mdbook;
extern crate env_logger;
extern crate tempdir;

use std::path::PathBuf;

use mdbook::loader::load_book;


#[test]
fn load_the_example_book() {
    let example_src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("book-example")
        .join("src");

    let book = load_book(example_src_dir).unwrap();
    println!("{:#?}", book);
}
