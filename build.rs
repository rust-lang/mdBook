// build.rs

use std::process::Command;
use std::env;
use std::path::Path;

fn main() {

    // Compile stylus stylesheet to css
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let theme_dir = Path::new(&manifest_dir).join("src/theme/");
    let stylus_dir = theme_dir.join("stylus/book.styl");

    if !Command::new("stylus")
            .arg(format!("{}", stylus_dir.to_str().unwrap()))
            .arg("--out")
            .arg(format!("{}", theme_dir.to_str().unwrap()))
            .arg("--use")
            .arg("nib")
            .status().unwrap()
            .success() {
        panic!("Stylus encoutered an error");
    }

}
