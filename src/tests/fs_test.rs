#[cfg(test)]

use std;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use utils;

#[test]
fn it_copies_data_file() {
    let dest_base = std::env::temp_dir().join("in tangles of old alleys");
    let dest_path = dest_base.join("book.css");

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

    if dest_base.exists() {
        fs::remove_dir_all(dest_base);
    }
}

#[test]
fn it_copies_data_by_pattern() {
    let dest_base = std::env::temp_dir().join("near the quays");

    if let Err(e) = utils::fs::copy_data("data/_html-template/**/*",
                                         "data/_html-template/",
                                         vec!["data/_html-template/_*"],
                                         &dest_base) {
        println!("Error: {:#?}", e);
        return;
    }

    assert!(dest_base.join("css").join("book.css").exists());
    assert!(!dest_base.join("_layouts").exists());

    if dest_base.exists() {
        fs::remove_dir_all(dest_base);
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

    utils::fs::clean_output_dir(&dest_base);

    assert!(dest_base.exists());
    assert!(dest_base.join(".dotfile").exists());
    assert!(!dest_base.join("door.html").exists());

    if dest_base.exists() {
        fs::remove_dir_all(dest_base);
    }
}
