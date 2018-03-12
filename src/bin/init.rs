use std::io;
use std::io::Write;
use mdbook::MDBook;
use mdbook::errors::Result;
use get_book_dir;

#[derive(StructOpt)]
pub struct InitArgs {
    #[structopt(long = "theme", help = "Copies the default theme into your source folder")]
    pub theme: bool,
    #[structopt(long = "force", help = "Skip confirmation prompts")] pub force: bool,
    #[structopt(help = "A directory for your book{n}(Defaults to Current Directory when omitted)")]
    pub dir: Option<String>,
}

// Init command implementation
pub fn execute(args: InitArgs) -> Result<()> {
    let book_dir = get_book_dir(args.dir);
    let mut builder = MDBook::init(&book_dir);

    // If flag `--theme` is present, copy theme to src
    if args.theme {
        // Skip this if `--force` is present
        if !args.force {
            // Print warning
            println!();
            println!(
                "Copying the default theme to {}",
                builder.config().book.src.display()
            );
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
