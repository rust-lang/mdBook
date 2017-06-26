use std::io;
use std::io::Write;
use clap::{ArgMatches, SubCommand, App};
use mdbook::MDBook;
use mdbook::errors::Result;
use get_book_dir;

// Init command implementation
pub fn init(args: &ArgMatches) -> Result<()> {

    let book_dir = get_book_dir(args);
    let mut book = MDBook::new(&book_dir);

    // Call the function that does the initialization
    book.init()?;

    // If flag `--theme` is present, copy theme to src
    if args.is_present("theme") {

        // Skip this if `--force` is present
        if !args.is_present("force") {
            // Print warning
            print!("\nCopying the default theme to {:?}", book.get_source());
            println!("could potentially overwrite files already present in that directory.");
            print!("\nAre you sure you want to continue? (y/n) ");

            // Read answer from user and exit if it's not 'yes'
            if !confirm() {
                println!("\nSkipping...\n");
                println!("All done, no errors...");
                ::std::process::exit(0);
            }
        }

        // Call the function that copies the theme
        book.copy_theme()?;
        println!("\nTheme copied.");

    }

    // Because of `src/book/mdbook.rs#L37-L39`, `dest` will always start with `root`
    let is_dest_inside_root = book.get_destination()
        .map(|p| p.starts_with(book.get_root()))
        .unwrap_or(false);

    if !args.is_present("force") && is_dest_inside_root {
        println!("\nDo you want a .gitignore to be created? (y/n)");

        if confirm() {
            book.create_gitignore();
            println!("\n.gitignore created.");
        }
    }

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

pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init")
        .about("Create boilerplate structure and files in the directory")
        // the {n} denotes a newline which will properly aligned in all help messages
        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'")
        .arg_from_usage("--theme 'Copies the default theme into your source folder'")
        .arg_from_usage("--force 'skip confirmation prompts'")
}
