extern crate mdbook;
#[macro_use]
extern crate clap;
extern crate log;
extern crate env_logger;
extern crate open;

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{App, ArgMatches, SubCommand, AppSettings};

#[cfg(feature = "serve")]
pub mod serve;
#[cfg(feature = "watch")]
pub mod watch;

use mdbook::MDBook;

const NAME: &'static str = "mdbook";

fn main() {
    env_logger::init().unwrap();

    // Create a list of valid arguments and sub-commands
    let matches = App::new(NAME)
                    .about("Create a book in form of a static website from markdown files")
                    .author("Mathieu David <mathieudavid@mathieudavid.org>")
                    // Get the version from our Cargo.toml using clap's crate_version!() macro
                    .version(&*format!("v{}", crate_version!()))
                    .setting(AppSettings::SubcommandRequired)
                    .after_help("For more information about a specific command, try `mdbook <command> --help`\nSource code for mdbook available at: https://github.com/azerupi/mdBook")
                    .subcommand(SubCommand::with_name("init")
                        .about("Create boilerplate structure and files in the directory")
                        // the {n} denotes a newline which will properly aligned in all help messages
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'")
                        .arg_from_usage("--theme 'Copies the default theme into your source folder'")
                        .arg_from_usage("--force 'skip confirmation prompts'"))
                    .subcommand(SubCommand::with_name("build")
                        .about("Build the book from the markdown files")
                        .arg_from_usage("-o, --open 'Open the compiled book in a web browser'")
                        .arg_from_usage("-d, --dest-dir=[dest-dir] 'The output directory for your book{n}(Defaults to ./book when omitted)'")
                        .arg_from_usage("--no-create 'Will not create non-existent files linked from SUMMARY.md'")
                        .arg_from_usage("--curly-quotes 'Convert straight quotes to curly quotes, except for those that occur in code blocks and code spans'")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'"))
                    .subcommand(SubCommand::with_name("watch")
                        .about("Watch the files for changes")
                        .arg_from_usage("-o, --open 'Open the compiled book in a web browser'")
                        .arg_from_usage("-d, --dest-dir=[dest-dir] 'The output directory for your book{n}(Defaults to ./book when omitted)'")
                        .arg_from_usage("--curly-quotes 'Convert straight quotes to curly quotes, except for those that occur in code blocks and code spans'")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'"))
                    .subcommand(SubCommand::with_name("serve")
                        .about("Serve the book at http://localhost:3000. Rebuild and reload on change.")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'")
                        .arg_from_usage("-d, --dest-dir=[dest-dir] 'The output directory for your book{n}(Defaults to ./book when omitted)'")
                        .arg_from_usage("--curly-quotes 'Convert straight quotes to curly quotes, except for those that occur in code blocks and code spans'")
                        .arg_from_usage("-p, --port=[port] 'Use another port{n}(Defaults to 3000)'")
                        .arg_from_usage("-w, --websocket-port=[ws-port] 'Use another port for the websocket connection (livereload){n}(Defaults to 3001)'")
                        .arg_from_usage("-i, --interface=[interface] 'Interface to listen on{n}(Defaults to localhost)'")
                        .arg_from_usage("-a, --address=[address] 'Address that the browser can reach the websocket server from{n}(Defaults to the interface address)'")
                        .arg_from_usage("-o, --open 'Open the book server in a web browser'"))
                    .subcommand(SubCommand::with_name("test")
                        .about("Test that code samples compile"))
                    .get_matches();

    // Check which subcomamnd the user ran...
    let res = match matches.subcommand() {
        ("init", Some(sub_matches)) => init(sub_matches),
        ("build", Some(sub_matches)) => build(sub_matches),
        #[cfg(feature = "watch")]
        ("watch", Some(sub_matches)) => watch::watch(sub_matches),
        #[cfg(feature = "serve")]
        ("serve", Some(sub_matches)) => serve::serve(sub_matches),
        ("test", Some(sub_matches)) => test(sub_matches),
        (_, _) => unreachable!(),
    };

    if let Err(e) = res {
        writeln!(&mut io::stderr(), "An error occured:\n{}", e).ok();
        ::std::process::exit(101);
    }
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


// Init command implementation
fn init(args: &ArgMatches) -> Result<(), Box<Error>> {

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


// Build command implementation
fn build(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);
    let book = MDBook::new(&book_dir).read_config()?;

    let mut book = match args.value_of("dest-dir") {
        Some(dest_dir) => book.with_destination(dest_dir),
        None => book,
    };

    if args.is_present("no-create") {
        book.create_missing = false;
    }

    if args.is_present("curly-quotes") {
        book = book.with_curly_quotes(true);
    }

    book.build()?;

    if let Some(d) = book.get_destination() {
        if args.is_present("open") {
            open(d.join("index.html"));
        }
    }

    Ok(())
}

fn test(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::new(&book_dir).read_config()?;

    book.test()?;

    Ok(())
}


fn get_book_dir(args: &ArgMatches) -> PathBuf {
    if let Some(dir) = args.value_of("dir") {
        // Check if path is relative from current dir, or absolute...
        let p = Path::new(dir);
        if p.is_relative() {
            env::current_dir().unwrap().join(dir)
        } else {
            p.to_path_buf()
        }
    } else {
        env::current_dir().unwrap()
    }
}

fn open<P: AsRef<OsStr>>(path: P) {
    if let Err(e) = open::that(path) {
        println!("Error opening web browser: {}", e);
    }
}
