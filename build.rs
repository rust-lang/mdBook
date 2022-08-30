use std::env;
use std::error::Error;
use syntect::dumps::dump_to_file;
use syntect::parsing::SyntaxSet;

pub fn main() -> Result<(), Box<dyn Error>> {
    let src_dir = format!(
        "{}/src/theme/syntaxes/",
        env::var("CARGO_MANIFEST_DIR").unwrap()
    );
    let dest = format!("{}/syntaxes.bin", env::var("OUT_DIR").unwrap());

    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    builder.add_from_folder(&src_dir, true)?;
    dump_to_file(&builder.build(), dest)?;

    Ok(())
}
