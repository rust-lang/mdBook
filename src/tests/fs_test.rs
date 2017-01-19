#![cfg(test)]

extern crate tempdir;

use std;
use std::fs::{self, File};
use std::io::Read;

use utils;
use utils::fs::copy_files_except_ext;

#[test]
fn it_copies_data_file() {
    let dest_base = std::env::temp_dir().join("in tangles of old alleys");
    let dest_path = dest_base.join("book.css");

    match utils::fs::copy_data_file("data/assets/_html-template/css/books.css", &dest_path) {
        Ok(_) => {},
        Err(e) => { println!("{:#?}", e); }
    }

    let mut file = match File::open(&dest_path) {
        Ok(f) => f,
        Err(_) => {
            println!("Failed to open {:?}", dest_path);
            return;
        },
    };

    let mut content = String::new();
    if let Err(_) = file.read_to_string(&mut content) {
        println!("Failed to read {:?}", dest_path);
        return;
    }

    assert!(content.as_str().contains("Open Sans"));

    if dest_base.exists() {
        match fs::remove_dir_all(dest_base) {
            Ok(_) => {},
            Err(e) => { println!("{:#?}", e); },
        }
    }
}

#[test]
fn it_copies_data_by_pattern() {
    let dest_base = std::env::temp_dir().join("near the quays");

    if let Err(e) = utils::fs::copy_data("data/assets/_html-template/**/*",
                                         "data/assets/_html-template/",
                                         vec!["data/assets/_html-template/_*"],
                                         &dest_base) {
        println!("Error: {:#?}", e);
        return;
    }

    assert!(dest_base.join("css").join("book.css").exists());
    assert!(!dest_base.join("_layouts").exists());

    if dest_base.exists() {
        match fs::remove_dir_all(dest_base) {
            Ok(_) => {},
            Err(e) => { println!("{:#?}", e); },
        }
    }
}

#[test]
fn it_doesnt_delete_toplevel_dotfiles() {
    let dest_base = std::env::temp_dir().join("with queer curls of fog");

    match utils::fs::create_with_str(&dest_base.join(".dotfile"), "that west winds tossed") {
        Err(e) => { println!("Error: {:#?}", e); return; },
        Ok(_) => {},
    }

    match utils::fs::create_with_str(&dest_base.join("door.html"), "<p>I entered, charmed</p>") {
        Err(e) => { println!("Error: {:#?}", e); return; },
        Ok(_) => {},
    }

    match utils::fs::clean_output_dir(&dest_base) {
        Ok(_) => {},
        Err(e) => { println!("{:#?}", e); }
    }

    assert!(dest_base.exists());
    assert!(dest_base.join(".dotfile").exists());
    assert!(!dest_base.join("door.html").exists());

    if dest_base.exists() {
        match fs::remove_dir_all(dest_base) {
            Ok(_) => {},
            Err(e) => { println!("{:#?}", e); },
        }
    }
}

#[test]
fn copy_files_except_ext_test() {
    let tmp = match tempdir::TempDir::new("") {
        Ok(t) => t,
        Err(_) => panic!("Could not create a temp dir"),
    };

    // Create a couple of files
    if let Err(_) = fs::File::create(&tmp.path().join("file.txt")) {
        panic!("Could not create file.txt")
    }
    if let Err(_) = fs::File::create(&tmp.path().join("file.md")) {
        panic!("Could not create file.md")
    }
    if let Err(_) = fs::File::create(&tmp.path().join("file.png")) {
        panic!("Could not create file.png")
    }
    if let Err(_) = fs::create_dir(&tmp.path().join("sub_dir")) {
        panic!("Could not create sub_dir")
    }
    if let Err(_) = fs::File::create(&tmp.path().join("sub_dir/file.png")) {
        panic!("Could not create sub_dir/file.png")
    }
    if let Err(_) = fs::create_dir(&tmp.path().join("sub_dir_exists")) {
        panic!("Could not create sub_dir_exists")
    }
    if let Err(_) = fs::File::create(&tmp.path().join("sub_dir_exists/file.txt")) {
        panic!("Could not create sub_dir_exists/file.txt")
    }

    // Create output dir
    if let Err(_) = fs::create_dir(&tmp.path().join("output")) {
        panic!("Could not create output")
    }
    if let Err(_) = fs::create_dir(&tmp.path().join("output/sub_dir_exists")) {
        panic!("Could not create output/sub_dir_exists")
    }

    match copy_files_except_ext(&tmp.path(), &tmp.path().join("output"), true, &["md"]) {
        Err(e) => panic!("Error while executing the function:\n{:?}", e),
        Ok(_) => {},
    }

    // Check if the correct files where created
    if !(&tmp.path().join("output/file.txt")).exists() {
        panic!("output/file.txt should exist")
    }
    if (&tmp.path().join("output/file.md")).exists() {
        panic!("output/file.md should not exist")
    }
    if !(&tmp.path().join("output/file.png")).exists() {
        panic!("output/file.png should exist")
    }
    if !(&tmp.path().join("output/sub_dir/file.png")).exists() {
        panic!("output/sub_dir/file.png should exist")
    }
    if !(&tmp.path().join("output/sub_dir_exists/file.txt")).exists() {
        panic!("output/sub_dir/file.png should exist")
    }

}
