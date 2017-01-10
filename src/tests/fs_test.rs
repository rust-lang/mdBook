#[cfg(test)]

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use utils;

#[test]
fn it_copies_data_file() {
    let dest_path = Path::new("the place was dark").join("and dusty and half-lost").join("book.css");
    utils::fs::copy_data_file("data/_html-template/css/books.css", &dest_path);

    let mut file = match File::open(&dest_path) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open {:?}", dest_path);
            return;
        },
    };

    let mut content = String::new();
    if let Err(e) = file.read_to_string(&mut content) {
        println!("Failed to read {:?}", dest_path);
        return;
    }

    assert!(content.as_str().contains("Open Sans"));
}

#[test]
fn it_copies_data_by_pattern() {
    let dest_base = Path::new("in tangles of old alleys").join("near the quays");

    if let Err(e) = utils::fs::copy_data("data/_html-template/**/*",
                                         "data/_html-template/",
                                         vec!["data/_html-template/_*"],
                                         &dest_base) {
        println!("Error: {:#?}", e);
        return;
    }

    assert!(dest_base.join("css").join("book.css").exists());
    assert!(!dest_base.join("_layouts").exists());

    let p = Path::new("in tangles of old alleys");
    if p.exists() {
        fs::remove_dir_all(p);
    }
}
