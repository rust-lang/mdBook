use std::io;
use std::io::Write;
use clap::{App, ArgMatches, SubCommand};
use mdbook::MDBook;
use mdbook::errors::Result;
use get_book_dir;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init")
        .about("Create boilerplate structure and files in the directory")
        // the {n} denotes a newline which will properly aligned in all help messages
        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory \
                         when omitted)'")
        .arg_from_usage("--theme 'Copies the default theme into your source folder'")
        .arg_from_usage("--force 'skip confirmation prompts'")
}

// Init command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut builder = MDBook::init(&book_dir);

    // If flag `--theme` is present, copy theme to src
    if args.is_present("theme") {
        // Skip this if `--force` is present
        if !args.is_present("force") {
            // Print warning
            print!("\nCopying the default theme to {}", builder.config().book.src.display());
            println!("This could potentially overwrite files already present in that directory.");
            print!("\nAre you sure you want to continue? (y/n) ");

            // Read answer from user and exit if it's not 'yes'
            if confirm() {
                builder.copy_theme(true);
            }
        }
    }

    println!("\nDo you want a .gitignore to be created? (y/n)");

    if confirm() {
        builder.create_gitignore(true);
    }

    builder.build()?;
    println!("\nAll done, no errors...");

    Ok(())
}

// Simple function that user comfirmation
fn confirm() -> bool {
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    match &*s.trim() {
        "Y" | "y" | "yes" | "Yes" => true,
        _ => false,
    }
}
