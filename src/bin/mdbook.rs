extern crate mdbook;
#[macro_use]
extern crate clap;
#[cfg(feature = "watch")]
extern crate notify;

use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{App, ArgMatches, SubCommand};

#[cfg(feature = "watch")]
use notify::Watcher;
#[cfg(feature = "watch")]
use std::sync::mpsc::channel;

use mdbook::MDBook;

const NAME: &'static str = "mdbook";

fn main() {
    // Create a list of valid arguments and sub-commands
    let matches = App::new(NAME)
                    .about("Create a book in form of a static website from markdown files")
                    .author("Mathieu David <mathieudavid@mathieudavid.org>")
                    // Get the version from our Cargo.toml using clap's crate_version!() macro
                    .version(&*format!("v{}", crate_version!()))
                    .subcommand_required(true)
                    .after_help("For more information about a specific command, try `mdbook <command> --help`")
                    .subcommand(SubCommand::with_name("init")
                        .about("Create boilerplate structure and files in the directory")
                        // the {n} denotes a newline which will properly aligned in all help messages
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when ommitted)'")
                        .arg_from_usage("--theme 'Copies the default theme into your source folder'")
                        .arg_from_usage("--force 'skip confirmation prompts'"))
                    .subcommand(SubCommand::with_name("build")
                        .about("Build the book from the markdown files")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when ommitted)'"))
                    .subcommand(SubCommand::with_name("watch")
                        .about("Watch the files for changes")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when ommitted)'"))
                    .get_matches();

    // Check which subcomamnd the user ran...
    let res = match matches.subcommand() {
        ("init", Some(sub_matches))  => init(sub_matches),
        ("build", Some(sub_matches)) => build(sub_matches),
        #[cfg(feature = "watch")]
        ("watch", Some(sub_matches)) => watch(sub_matches),
        (_, _)                       => unreachable!()
    };

    if let Err(e) = res {
        writeln!(&mut io::stderr(), "An error occured:\n{}", e).ok();
    }
}

fn confirm() -> bool {
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    match &*s.trim() {
        "Y" | "y" | "yes" | "Yes" => true,
        _ => false
    }
}

fn init(args: &ArgMatches) -> Result<(), Box<Error>> {

    let book_dir = get_book_dir(args);
    let mut book = MDBook::new(&book_dir);

    // Call the function that does the initialization
    try!(book.init());

    // If flag `--theme` is present, copy theme to src
    if args.is_present("theme") {

        // Skip this id `--force` is present
        if !args.is_present("force") {
            // Print warning
            print!("\nCopying the default theme to {:?}", book.get_src());
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
        try!(book.copy_theme());
        println!("\nTheme copied.");

    }

    println!("\nAll done, no errors...");

    Ok(())
}

fn build(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::new(&book_dir).read_config();

    try!(book.build());

    Ok(())
}

#[cfg(feature = "watch")]
fn watch(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);
    let book = MDBook::new(&book_dir).read_config();

    // Create a channel to receive the events.
     let (tx, rx) = channel();

     let w: Result<notify::RecommendedWatcher, notify::Error> = notify::Watcher::new(tx);

     match w {
         Ok(mut watcher) => {

             watcher.watch(book.get_src()).unwrap();

             loop {
                 match rx.recv() {
                     Ok(event) => {
                         if let Some(path) = event.path {
                             println!("File changed: {:?}\nBuilding book...\n", path);
                             try!(build(args));
                             println!("");
                             // Hack to prevent receiving the event 4 times, probably a bug in notify
                             return watch(args);
                         } else {
                             continue;
                         }
                     },
                     Err(e) => {
                         println!("An error occured: {:?}", e);
                     }
                 }
             }

         },
         Err(e) => {
             println!("Error while trying to watch the files:\n\n\t{:?}", e);
             ::std::process::exit(0);
         }
     }

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
