extern crate mdbook;
#[macro_use]
extern crate clap;
extern crate log;
extern crate env_logger;
extern crate open;

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
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{App, ArgMatches, SubCommand, AppSettings};

// Uses for the Watch feature
#[cfg(feature = "watch")]
use notify::Watcher;
#[cfg(feature = "watch")]
use std::time::Duration;
#[cfg(feature = "watch")]
use std::sync::mpsc::channel;


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
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'"))
                    .subcommand(SubCommand::with_name("watch")
                        .about("Watch the files for changes")
                        .arg_from_usage("-o, --open 'Open the compiled book in a web browser'")
                        .arg_from_usage("-d, --dest-dir=[dest-dir] 'The output directory for your book{n}(Defaults to ./book when omitted)'")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'"))
                    .subcommand(SubCommand::with_name("serve")
                        .about("Serve the book at http://localhost:3000. Rebuild and reload on change.")
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'")
                        .arg_from_usage("-d, --dest-dir=[dest-dir] 'The output directory for your book{n}(Defaults to ./book when omitted)'")
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
    let mut book = MDBook::new(&book_dir);

    // Call the function that does the initialization
    try!(book.init());

    // If flag `--theme` is present, copy theme to src
    if args.is_present("theme") {

        // Skip this if `--force` is present
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

    // Because of `src/book/mdbook.rs#L37-L39`, `dest` will always start with `root`
    let is_dest_inside_root = book.get_dest().starts_with(book.get_root());

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
    let book = MDBook::new(&book_dir).read_config();

    let mut book = match args.value_of("dest-dir") {
        Some(dest_dir) => book.set_dest(Path::new(dest_dir)),
        None => book
    };

    if args.is_present("no-create") {
        book.create_missing = false;
    }

    try!(book.build());

    if args.is_present("open") {
        open(book.get_dest().join("index.html"));
    }

    Ok(())
}


// Watch command implementation
#[cfg(feature = "watch")]
fn watch(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);
    let book = MDBook::new(&book_dir).read_config();

    let mut book = match args.value_of("dest-dir") {
        Some(dest_dir) => book.set_dest(Path::new(dest_dir)),
        None => book
    };

    if args.is_present("open") {
        try!(book.build());
        open(book.get_dest().join("index.html"));
    }

    trigger_on_change(&mut book, |path, book| {
        println!("File changed: {:?}\nBuilding book...\n", path);
        if let Err(e) = book.build() {
            println!("Error while building: {:?}", e);
        }
        println!("");
    });

    Ok(())
}


// Watch command implementation
#[cfg(feature = "serve")]
fn serve(args: &ArgMatches) -> Result<(), Box<Error>> {
    const RELOAD_COMMAND: &'static str = "reload";

    let book_dir = get_book_dir(args);
    let book = MDBook::new(&book_dir).read_config();

    let mut book = match args.value_of("dest-dir") {
        Some(dest_dir) => book.set_dest(Path::new(dest_dir)),
        None => book
    };

    let port = args.value_of("port").unwrap_or("3000");
    let ws_port = args.value_of("ws-port").unwrap_or("3001");
    let interface = args.value_of("interface").unwrap_or("localhost");
    let public_address = args.value_of("address").unwrap_or(interface);
    let open_browser = args.is_present("open");

    let address = format!("{}:{}", interface, port);
    let ws_address = format!("{}:{}", interface, ws_port);

    book.set_livereload(format!(r#"
        <script type="text/javascript">
            var socket = new WebSocket("ws://{}:{}");
            socket.onmessage = function (event) {{
                if (event.data === "{}") {{
                    socket.close();
                    location.reload(true); // force reload from server (not from cache)
                }}
            }};

            window.onbeforeunload = function() {{
                socket.close();
            }}
        </script>
    "#, public_address, ws_port, RELOAD_COMMAND).to_owned());

    try!(book.build());

    let staticfile = staticfile::Static::new(book.get_dest());
    let iron = iron::Iron::new(staticfile);
    let _iron = iron.http(&*address).unwrap();

    let ws_server = ws::WebSocket::new(|_| {
        |_| {
            Ok(())
        }
    }).unwrap();

    let broadcaster = ws_server.broadcaster();

    std::thread::spawn(move || {
        ws_server.listen(&*ws_address).unwrap();
    });

    println!("\nServing on {}", address);

    if open_browser {
        open(format!("http://{}", address));
    }

    trigger_on_change(&mut book, move |path, book| {
        println!("File changed: {:?}\nBuilding book...\n", path);
        match book.build() {
            Err(e) => println!("Error while building: {:?}", e),
            _ => broadcaster.send(RELOAD_COMMAND).unwrap(),
        }
        println!("");
    });

    Ok(())
}


fn test(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::new(&book_dir).read_config();

    try!(book.test());

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


// Calls the closure when a book source file is changed. This is blocking!
#[cfg(feature = "watch")]
fn trigger_on_change<F>(book: &mut MDBook, closure: F) -> ()
    where F: Fn(&Path, &mut MDBook) -> ()
{
    use notify::RecursiveMode::*;
    use notify::DebouncedEvent::*;

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
        Ok(w) => w,
        Err(e) => {
            println!("Error while trying to watch the files:\n\n\t{:?}", e);
            ::std::process::exit(0);
        }
    };

    // Add the source directory to the watcher
    if let Err(e) = watcher.watch(book.get_src(), Recursive) {
        println!("Error while watching {:?}:\n    {:?}", book.get_src(), e);
        ::std::process::exit(0);
    };

    // Add the book.{json,toml} file to the watcher if it exists, because it's not
    // located in the source directory
    if watcher.watch(book.get_root().join("book.json"), NonRecursive).is_err() {
        // do nothing if book.json is not found
    }
    if watcher.watch(book.get_root().join("book.toml"), NonRecursive).is_err() {
        // do nothing if book.toml is not found
    }

    println!("\nListening for changes...\n");

    loop {
        match rx.recv() {
            Ok(event) => match event {
                NoticeWrite(path) |
                NoticeRemove(path) |
                Create(path) |
                Write(path) |
                Remove(path) |
                Rename(_, path) => {
                    closure(&path, book);
                }
                _ => {}
            },
            Err(e) => {
                println!("An error occured: {:?}", e);
            },
        }
    }
}
