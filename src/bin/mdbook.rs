#[macro_use]
extern crate mdbook;
#[macro_use]
extern crate clap;
#[macro_use]
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
use std::fs;
use std::error::Error;
use std::ffi::OsStr;
use std::io::{self, Write, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{App, ArgMatches, SubCommand, AppSettings};

// Uses for the Watch feature
#[cfg(feature = "watch")]
use notify::Watcher;
#[cfg(feature = "watch")]
use std::time::Duration;
#[cfg(feature = "watch")]
use std::sync::mpsc::channel;

use mdbook::MDBook;
use mdbook::renderer::{Renderer, HtmlHandlebars};
use mdbook::book::toc::TocItem;
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
                        .arg_from_usage("[dir] 'A directory for your book{n}(Defaults to Current Directory when omitted)'")
                        .arg_from_usage("--copy-assets 'Copies the default assets (css, layout template, etc.) into your project folder'")
                        .arg_from_usage("--force 'skip confirmation prompts'"))
                    .subcommand(SubCommand::with_name("build")
                        .about("Build the book from the markdown files")
                        .arg_from_usage("-o, --open 'Open the compiled book in a web browser'")
                        .arg_from_usage("-d, --dest-dir=[dest-dir] 'The output directory for your book{n}(Defaults to ./book when omitted)'")
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
                        .arg_from_usage("-a, --address=[address] 'Address that the browser can reach the websocket server from{n}(Defaults to the interface addres)'")
                        .arg_from_usage("-o, --open 'Open the compiled book in a web browser'"))
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

/// Init command implementation
///
/// It creates some boilerplate files and directories to get you started with your book.
///
/// ```text
/// thebook
/// ├── book.toml
/// └── src
///     ├── SUMMARY.md
///     ├── first-chapter.md
///     ├── glossary.md
///     └── introduction.md
/// ```
///
/// It copies the embedded starter book as stored in data/book-init.
fn init(args: &ArgMatches) -> Result<(), Box<Error>> {
    debug!("[fn]: init");

    let book_dir = get_book_dir(args);

    if !book_dir.exists() {
        fs::create_dir_all(&book_dir).unwrap();
        info!("{:?} created", &book_dir);
    }

    try!(utils::fs::copy_data("data/book-init/*",
                              "data/book-init/",
                              vec![],
                              &book_dir));

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
        try!(utils::fs::copy_data("data/assets/**/*",
                                  "data/assets/",
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

    debug!("[*]: init done");
    Ok(())
}

// Build command implementation
fn build(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);

    let mut dest_base: Option<PathBuf> = None;
    if let Some(p) = args.value_of("dest-dir") {
        dest_base = Some(PathBuf::from(p));
    }

    // TODO select render format intent when we acutally have different renderers
    let renderer = HtmlHandlebars::new();
    let book_project: MDBook = try!(renderer.build(&book_dir, &dest_base));

    if args.is_present("open") {
        open(book_project.get_dest_base().join("index.html"));
    }

    Ok(())
}

// Watch command implementation
#[cfg(feature = "watch")]
fn watch(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);

    let mut dest_base: Option<PathBuf> = None;
    if let Some(p) = args.value_of("dest-dir") {
        dest_base = Some(PathBuf::from(p));
    }

    let renderer = HtmlHandlebars::new();
    let mut book_project: MDBook = try!(renderer.build(&book_dir, &dest_base));

    if args.is_present("open") {
        open(book_project.get_dest_base().join("index.html"));
    }

    trigger_on_change(&mut book_project, |path, _| {
        println!("File changed: {:?}\nBuilding book...\n", path);
        // TODO select render format intent when we acutally have different renderers
        let renderer = HtmlHandlebars::new();
        match renderer.build(&book_dir, &dest_base) {
            Err(e) => println!("Error while building: {:?}", e),
            _ => {},
        }
        println!("");
    });

    println!("watch");
    Ok(())
}

// Serve command implementation
#[cfg(feature = "serve")]
fn serve(args: &ArgMatches) -> Result<(), Box<Error>> {
    const RELOAD_COMMAND: &'static str = "reload";

    let book_dir = get_book_dir(args);

    let mut dest_base: Option<PathBuf> = None;
    if let Some(p) = args.value_of("dest-dir") {
        dest_base = Some(PathBuf::from(p));
    }

    let mut book = MDBook::new(&book_dir);

    book.read_config();

    if let Some(p) = dest_base {
        book.set_dest_base(&p);
    }

    book.parse_books();
    book.link_translations();

    let port = args.value_of("port").unwrap_or("3000");
    let ws_port = args.value_of("ws-port").unwrap_or("3001");
    let interface = args.value_of("interface").unwrap_or("localhost");
    let public_address = args.value_of("address").unwrap_or(interface);
    let open_browser = args.is_present("open");

    let address = format!("{}:{}", interface, port);
    let ws_address = format!("{}:{}", interface, ws_port);

    book.livereload_script = Some(format!(r#"
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
    "#, public_address, ws_port, RELOAD_COMMAND));

    let renderer = HtmlHandlebars::new();
    try!(renderer.render(&book));

    let staticfile = staticfile::Static::new(book.get_dest_base());
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
        let renderer = HtmlHandlebars::new();
        match renderer.render(&book) {
            Err(e) => println!("Error while building: {:?}", e),
            _ => broadcaster.send(RELOAD_COMMAND).unwrap(),
        }
        println!("");
    });

    Ok(())
}

/// Run the code examples in the book's chapters as tests with rustdoc
fn test(args: &ArgMatches) -> Result<(), Box<Error>> {
    let book_dir = get_book_dir(args);
    let mut proj = MDBook::new(&book_dir);
    proj.read_config();
    proj.parse_books();

    for (_, book) in proj.translations.iter() {
        for item in book.toc.iter() {
            match *item {
                TocItem::Numbered(ref i) |
                TocItem::Unnumbered(ref i) |
                TocItem::Unlisted(ref i) => {
                    if let Some(p) = i.chapter.get_src_path() {
                        let path = book.config.get_src().join(&p);

                        println!("[*]: Testing file: {:?}", path);

                        let output_result = Command::new("rustdoc")
                            .arg(&path)
                            .arg("--test")
                            .output();
                        let output = try!(output_result);

                        if !output.status.success() {
                            return Err(Box::new(io::Error::new(ErrorKind::Other, format!(
                                "{}\n{}",
                                String::from_utf8_lossy(&output.stdout),
                                String::from_utf8_lossy(&output.stderr)))) as Box<Error>);
                        }
                    }
                },
                TocItem::Spacer => {},
            }
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
            ::std::process::exit(2);
        }
    };

    // Add the source directory to the watcher
    if let Err(e) = watcher.watch(book.get_src_base(), Recursive) {
        println!("Error while watching {:?}:\n    {:?}", book.get_src_base(), e);
        ::std::process::exit(2);
    };

    // Add the book.{json,toml} file to the watcher if it exists, because it's not
    // located in the source directory
    if let Err(_) = watcher.watch(book.get_project_root().join("book.json"), NonRecursive) {
        // do nothing if book.json is not found
    }
    if let Err(_) = watcher.watch(book.get_project_root().join("book.toml"), NonRecursive) {
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
