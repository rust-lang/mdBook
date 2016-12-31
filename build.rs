// build.rs

extern crate includedir_codegen;

use includedir_codegen::Compression;

use std::process::Command;
use std::env;
use std::path::Path;

fn main() {

    includedir_codegen::start("FILES")
        .dir("data", Compression::Gzip)
        .build("data.rs")
        .unwrap();

    // TODO this using cargo as a Makefile. This is only for development, it
    // doesn't have to be part of the production auto-build. Use either a
    // Makefile or an npm command if stylus comes from npm anyway.

    if let Ok(_) = env::var("CARGO_FEATURE_REGENERATE_CSS") {

        // Compile stylus stylesheet to css
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        let template_dir = Path::new(&manifest_dir).join("data/html-template/");
        let stylus_dir = template_dir.join("_stylus/book.styl");

        if !Command::new("stylus")
                .arg(format!("{}", stylus_dir.to_str().unwrap()))
                .arg("--out")
                .arg(format!("{}", template_dir.to_str().unwrap()))
                .arg("--use")
                .arg("nib")
                .status().unwrap()
                .success() {
            panic!("Stylus encoutered an error");
        }
    }

}
