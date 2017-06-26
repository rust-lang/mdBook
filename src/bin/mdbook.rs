extern crate mdbook;
#[macro_use]
extern crate clap;
extern crate log;
extern crate env_logger;
extern crate open;

use std::env;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use clap::{App, ArgMatches, AppSettings};

pub mod build;
pub mod init;
pub mod test;
#[cfg(feature = "serve")]
pub mod serve;
#[cfg(feature = "watch")]
pub mod watch;

const NAME: &'static str = "mdbook";

fn main() {
    env_logger::init().unwrap();

    // Create a list of valid arguments and sub-commands
    let app = App::new(NAME)
                .about("Create a book in form of a static website from markdown files")
                .author("Mathieu David <mathieudavid@mathieudavid.org>")
                // Get the version from our Cargo.toml using clap's crate_version!() macro
                .version(concat!("v",crate_version!()))
                .setting(AppSettings::SubcommandRequired)
                .after_help("For more information about a specific command, try `mdbook <command> --help`\nSource code for mdbook available at: https://github.com/azerupi/mdBook")
                .subcommand(init::make_subcommand())
                .subcommand(build::make_subcommand())
                .subcommand(test::make_subcommand());

    #[cfg(feature = "watch")]
    let app = app.subcommand(watch::make_subcommand());
    #[cfg(feature = "serve")]
    let app = app.subcommand(serve::make_subcommand());

    // Check which subcomamnd the user ran...
    let res = match app.get_matches().subcommand() {
        ("init", Some(sub_matches)) => init::init(sub_matches),
        ("build", Some(sub_matches)) => build::build(sub_matches),
        #[cfg(feature = "watch")]
        ("watch", Some(sub_matches)) => watch::watch(sub_matches),
        #[cfg(feature = "serve")]
        ("serve", Some(sub_matches)) => serve::serve(sub_matches),
        ("test", Some(sub_matches)) => test::test(sub_matches),
        (_, _) => unreachable!(),
    };

    if let Err(e) = res {
        writeln!(&mut io::stderr(), "An error occured:\n{}", e).ok();
        ::std::process::exit(101);
    }
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
