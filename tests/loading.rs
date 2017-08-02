//! Integration tests for loading a book into memory

extern crate mdbook;
extern crate env_logger;

use std::path::PathBuf;

use mdbook::loader::load_book;


#[test]
fn load_the_example_book() {
    env_logger::init().ok();

    let example_src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("book-example")
        .join("src");

    let book = load_book(example_src_dir).unwrap();
    println!("{:#?}", book);
}
