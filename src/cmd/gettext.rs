use crate::cmd::xgettext::extract_paragraphs;
use crate::get_book_dir;
use crate::utils;
use anyhow::anyhow;
use anyhow::Context;
use clap::{arg, App, Arg, ArgMatches};
use mdbook::book::Chapter;
use mdbook::BookItem;
use mdbook::MDBook;
use polib::catalog::Catalog;
use polib::po_file::parse;
use std::path::Path;

// Create clap subcommand arguments
pub fn make_subcommand<'help>() -> App<'help> {
    App::new("gettext")
        .about("Output translated book")
        .arg(
            Arg::new("dest-dir")
                .short('d')
                .long("dest-dir")
                .value_name("dest-dir")
                .help(
                    "Output directory for the translated book{n}\
                     Relative paths are interpreted relative to the book's root directory{n}\
                     If omitted, mdBook defaults to `./src/xx` where `xx` is the language of the PO file."
                ),
        )
        .arg(arg!(<po> "PO file to generate translation for"))
        .arg(arg!([dir]
            "Root directory for the book{n}\
            (Defaults to the Current Directory when omitted)"
        ))
}

fn translate(text: &str, catalog: &Catalog) -> String {
    let mut output = String::with_capacity(text.len());
    let mut current_lineno = 1;

    for (lineno, paragraph) in extract_paragraphs(text) {
        // Fill in blank lines between paragraphs. This is
        // important for code blocks where blank lines can
        // be significant.
        while current_lineno < lineno {
            output.push('\n');
            current_lineno += 1;
        }
        current_lineno += paragraph.lines().count();

        let translated = catalog
            .find_message(paragraph)
            .and_then(|msg| msg.get_msgstr().ok())
            .filter(|msgstr| !msgstr.is_empty())
            .map(|msgstr| msgstr.as_str())
            .unwrap_or(paragraph);
        output.push_str(translated);
        output.push('\n');
    }

    output
}

// Gettext command implementation
pub fn execute(args: &ArgMatches) -> mdbook::errors::Result<()> {
    let book_dir = get_book_dir(args);
    let book = MDBook::load(&book_dir)?;

    let po_file = Path::new(args.value_of("po").unwrap());
    let lang = po_file
        .file_stem()
        .ok_or_else(|| anyhow!("Could not determine language from PO file {:?}", po_file))?;
    let catalog = parse(po_file)
        .map_err(|err| anyhow!(err.to_string()))
        .with_context(|| format!("Could not parse PO file {:?}", po_file))?;
    let dest_dir = book.root.join(match args.value_of("dest-dir") {
        Some(path) => path.into(),
        None => Path::new(&book.config.book.src).join(lang),
    });

    let summary_path = book_dir.join(&book.config.book.src).join("SUMMARY.md");
    let summary = std::fs::read_to_string(&summary_path)?;
    utils::fs::write_file(
        &dest_dir,
        "SUMMARY.md",
        translate(&summary, &catalog).as_bytes(),
    )?;

    for item in book.iter() {
        if let BookItem::Chapter(Chapter {
            content,
            path: Some(path),
            ..
        }) = item
        {
            let output = translate(content, &catalog);
            utils::fs::write_file(&dest_dir, path, output.as_bytes())?;
        }
    }

    Ok(())
}
