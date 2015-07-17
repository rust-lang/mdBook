extern crate mdbook;
extern crate getopts;
use std::env;

use mdbook::MDBook;

const NAME: &'static str = "mdbook";
const VERSION: &'static str = "0.0.1";

#[derive(Clone)]
struct Subcommand {
    name: &'static str,
    help: &'static str,
    exec: fn(args: Vec<String>)
}


// All subcommands
static SUBCOMMANDS: &'static [Subcommand] = &[
    Subcommand{ name: "init", help: " Create boilerplate structure and files in the directory", exec: init },
    Subcommand{ name: "build", help: "Build the book from the markdown files", exec: build },
    Subcommand{ name: "watch", help: "Watch the files for changes", exec: watch },
];


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut subcommand: Option<Subcommand> = None;

    if args.len() > 1 {
        // Check if one of the subcommands match
        for command in SUBCOMMANDS {
            if args[1] == command.name {
                subcommand = Some(command.clone());
            }
        }
    }

    match subcommand {
        None => no_subcommand(args),
        Some(command) => (command.exec)(args),
    }
}


fn no_subcommand(args: Vec<String>) {
    let mut opts = getopts::Options::new();

    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("", "version", "output version information and exit");

    let usage = opts.usage("Create a book in form of a static website from markdown files");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            println!("Try `{} --help` for more information", NAME);
            return;
        },
    };

    if matches.opt_present("version") {
        println!("{} {}", NAME, VERSION);
    } else {
        if !matches.opt_present("version") && args.len() > 0 {
            print!("Try again, `{0}", NAME);
            for index in 1..args.len() {
                print!(" {}", args[index]);
            }
            print!("` is not a valid command... ");
            println!("\n");
        }
        help(&usage);
    }
}

fn help(usage: &String) {

    println!("{0} {1} \n", NAME, VERSION);
    println!("Usage:");
    println!("    {0} <command> [<args>...]\n    {0} [options]\n", NAME);
    println!("{0}", usage);
    println!("Commands:");
    for subcommand in SUBCOMMANDS {
        println!("    {0}               {1}", subcommand.name, subcommand.help);
    }
    println!("");
    println!("For more information about a specific command, try `mdbook <command> --help`");
}

fn init(args: Vec<String>) {
    let mut opts = getopts::Options::new();

    opts.optflag("h", "help", "display this help and exit");

    let usage = opts.usage("Creates a skeleton structure and some boilerplate files to start with");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            println!("Try `{} --help` for more information", NAME);
            return;
        },
    };

    if matches.opt_present("help") {
        println!("{}", usage);
        return;
    }

    let dir = if args.len() <= 2 {
        std::env::current_dir().unwrap()
    } else {
        std::env::current_dir().unwrap().join(&args[2])
    };

    let book = MDBook::new(&dir);

    if let Err(e) = book.init() {
        println!("Error: {}", e);
    }
}

fn build(args: Vec<String>) {
    let mut opts = getopts::Options::new();

    opts.optflag("h", "help", "display this help and exit");

    let usage = opts.usage("Build the book from the markdown files");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            println!("Try `{} --help` for more information", NAME);
            return;
        },
    };

    if matches.opt_present("help") {
        println!("{}", usage);
    }

    let dir = if args.len() <= 2 {
        std::env::current_dir().unwrap()
    } else {
        std::env::current_dir().unwrap().join(&args[2])
    };
    
    let mut book = MDBook::new(&dir);

    if let Err(e) = book.build() {
        println!("Error: {}", e);
    }
}

fn watch(args: Vec<String>) {
    let mut opts = getopts::Options::new();

    opts.optflag("h", "help", "display this help and exit");

    let usage = opts.usage("Watch the files for changes");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            println!("Try `{} --help` for more information", NAME);
            return;
        },
    };

    if matches.opt_present("help") {
        println!("{}", usage);
    }
}
