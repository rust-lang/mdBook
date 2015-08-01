extern crate mdbook;
#[macro_use]
extern crate clap;

use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{App, ArgMatches, SubCommand};

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
                        // the {n} denotes a newline which will properly aligned in all help
                        // messages
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when ommitted)'"))
                    .subcommand(SubCommand::with_name("build")
                        .about("Build the book from the markdown files")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when ommitted)'"))
                    .subcommand(SubCommand::with_name("watch")
                        .about("Watch the files for changes"))
                    .get_matches();

    // Check which subcomamnd the user ran...
    let res = match matches.subcommand() {
        ("init", Some(sub_matches))  => init(sub_matches),
        ("build", Some(sub_matches)) => build(sub_matches),
        ("watch", _)                 => unimplemented!(),
        (_, _)                       => unreachable!()
    };

    if let Err(e) = res {
        writeln!(&mut io::stderr(), "Error: {}", e).ok();
    }
}

fn init(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);
    let book = MDBook::new(&book_dir);

    book.init()
}

fn build(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::new(&book_dir);

    book.build()
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
