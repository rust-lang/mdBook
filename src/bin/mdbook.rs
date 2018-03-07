extern crate chrono;
#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate mdbook;
extern crate open;

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::io::Write;
use clap::{App, AppSettings, ArgMatches};
use chrono::Local;
use log::LevelFilter;
use env_logger::Builder;
use mdbook::utils;

pub mod build;
pub mod clean;
pub mod init;
pub mod test;
#[cfg(feature = "serve")]
pub mod serve;
#[cfg(feature = "watch")]
pub mod watch;

const NAME: &'static str = "mdbook";

fn main() {
    init_logger();

    // Create a list of valid arguments and sub-commands
    let app = App::new(NAME)
                .about("Create a book in form of a static website from markdown files")
                .author("Mathieu David <mathieudavid@mathieudavid.org>")
                // Get the version from our Cargo.toml using clap's crate_version!() macro
                .version(concat!("v",crate_version!()))
                .setting(AppSettings::ArgRequiredElseHelp)
                .after_help("For more information about a specific command, \
                             try `mdbook <command> --help`\n\
                             Source code for mdbook available \
                             at: https://github.com/rust-lang-nursery/mdBook")
                .subcommand(init::make_subcommand())
                .subcommand(build::make_subcommand())
                .subcommand(test::make_subcommand())
                .subcommand(clean::make_subcommand());

    #[cfg(feature = "watch")]
    let app = app.subcommand(watch::make_subcommand());
    #[cfg(feature = "serve")]
    let app = app.subcommand(serve::make_subcommand());

    // Check which subcomamnd the user ran...
    let res = match app.get_matches().subcommand() {
        ("init", Some(sub_matches)) => init::execute(sub_matches),
        ("build", Some(sub_matches)) => build::execute(sub_matches),
        ("clean", Some(sub_matches)) => clean::execute(sub_matches),
        #[cfg(feature = "watch")]
        ("watch", Some(sub_matches)) => watch::execute(sub_matches),
        #[cfg(feature = "serve")]
        ("serve", Some(sub_matches)) => serve::execute(sub_matches),
        ("test", Some(sub_matches)) => test::execute(sub_matches),
        (_, _) => unreachable!(),
    };

    if let Err(e) = res {
        utils::log_backtrace(&e);

        ::std::process::exit(101);
    }
}

fn init_logger() {
    let mut builder = Builder::new();

    builder.format(|formatter, record| {
        writeln!(
            formatter,
            "{} [{}] ({}): {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    });

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse(&var);
    } else {
        // if no RUST_LOG provided, default to logging at the Info level
        builder.filter(None, LevelFilter::Info);
        // Filter extraneous html5ever not-implemented messages
        builder.filter(Some("html5ever"), LevelFilter::Error);
    }

    builder.init();
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
        env::current_dir().expect("Unable to determine the current directory")
    }
}

fn open<P: AsRef<OsStr>>(path: P) {
    if let Err(e) = open::that(path) {
        error!("Error opening web browser: {}", e);
    }
}
