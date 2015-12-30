// build.rs
extern crate sass_rs;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Write;

use sass_rs::sass_context::SassFileContext;

fn main() {
    if let Ok(_) = env::var("CARGO_FEATURE_REGENERATE_CSS") {
        // Compile stylus stylesheet to css
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        let theme_dir = Path::new(&manifest_dir).join("src/theme/");
        let scss_index_file = theme_dir.join("stylus/book.scss");

        let css = SassFileContext::new(&scss_index_file.to_string_lossy())
                                 .compile()
                                 .expect("error compiling SCSS files");
        let mut output_file = File::create(&theme_dir.join("book.css"))
                                  .expect("error creating CSS file");
        output_file.write_all(&mut css.as_bytes()).expect("error writing CSS to file");
    }
}
