#[macro_use]
extern crate mdbook;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;

// Dependencies for the Watch feature
#[cfg(feature = "watch")]
extern crate notify;
#[cfg(feature = "watch")]
extern crate time;
#[cfg(feature = "watch")]
extern crate crossbeam;

// Dependencies for the Serve feature
#[cfg(feature = "serve")]
extern crate iron;
#[cfg(feature = "serve")]
extern crate staticfile;
#[cfg(feature = "serve")]
extern crate ws;

use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{App, ArgMatches, SubCommand, AppSettings};

use mdbook::MDBook;
use mdbook::renderer::{Renderer, HtmlHandlebars};
use mdbook::utils;

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
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when ommitted)'")
                        .arg_from_usage("--copy-assets 'Copies the default assets (css, layout template, etc.) into your project folder'")
                        .arg_from_usage("--force 'skip confirmation prompts'"))
                    .subcommand(SubCommand::with_name("build")
                        .about("Build the book from the markdown files")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when ommitted)'"))
                    .subcommand(SubCommand::with_name("watch")
                        .about("Watch the files for changes")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when ommitted)'"))
                    .subcommand(SubCommand::with_name("serve")
                        .about("Serve the book at http://localhost:3000. Rebuild and reload on change.")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when ommitted)'")
                        .arg_from_usage("-p, --port=[port] 'Use another port{n}(Defaults to 3000)'")
                        .arg_from_usage("-w, --websocket-port=[ws-port] 'Use another port for the websocket connection (livereload){n}(Defaults to 3001)'")
                        .arg_from_usage("-i, --interface=[interface] 'Interface to listen on{n}(Defaults to localhost)'")
                        .arg_from_usage("-a, --address=[address] 'Address that the browser can reach the websocket server from{n}(Defaults to the interface addres)'"))
                    .subcommand(SubCommand::with_name("test")
                        .about("Test that code samples compile"))
                    .get_matches();

    // Check which subcomamnd the user ran...
    let res = match matches.subcommand() {
        ("init", Some(sub_matches)) => init(sub_matches),
        ("build", Some(sub_matches)) => build(sub_matches),
        #[cfg(feature = "watch")]
        ("watch", Some(sub_matches)) => watch(sub_matches),
        #[cfg(feature = "serve")]
        ("serve", Some(sub_matches)) => serve(sub_matches),
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
    let mut book_project = MDBook::new(&book_dir);

    book_project.read_config();
    book_project.parse_books();

    // If flag `--copy-assets` is present, copy embedded assets to project root
    if args.is_present("copy-assets") {

        // Skip this if `--force` is present
        if book_project.get_project_root().join("assets").exists() && !args.is_present("force") {
            // Print warning
            println!("\nCopying the default assets to {:?}", book_project.get_project_root());
            println!("This will overwrite files already present in that directory.");
            print!("Are you sure you want to continue? (y/n) ");

            // Read answer from user and exit if it's not 'yes'
            if !confirm() {
                println!("\nSkipping...\n");
                println!("All done, no errors...");
                ::std::process::exit(0);
            }
        }

        // Copy the assets
        try!(utils::fs::copy_data("data/**/*",
                                  "data/",
                                  vec![],
                                  &book_project.get_project_root().join("assets")));

        println!("\nAssets copied.");

    }

    if !args.is_present("force") {
        println!("\nDo you want a .gitignore to be created? (y/n)");

        if confirm() {
            utils::fs::create_gitignore(&book_project);
            println!("\n.gitignore created.");
        }
    }

    println!("\nAll done, no errors...");

    Ok(())
}

// Build command implementation
fn build(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);

    // TODO figure out render format intent when we acutally have different renderers

    let renderer = HtmlHandlebars::new();
    try!(renderer.build(&book_dir));

    Ok(())
}

// Watch command implementation
#[cfg(feature = "watch")]
fn watch(args: &ArgMatches) -> Result<(), Box<Error>> {
    // TODO watch
    println!("watch");
    Ok(())
}

// Serve command implementation
#[cfg(feature = "serve")]
fn serve(args: &ArgMatches) -> Result<(), Box<Error>> {
    // TODO serve
    println!("serve");
    Ok(())
}

fn test(args: &ArgMatches) -> Result<(), Box<Error>> {
    // TODO test
    println!("test");
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
