extern crate iron;
extern crate staticfile;
extern crate ws;

use std;
use std::path::Path;
use std::error::Error;
use self::iron::{Iron, AfterMiddleware, IronResult, IronError, Request, Response, status, Set, Chain};
use clap::ArgMatches;
use mdbook::MDBook;

use {get_book_dir, open, trigger_on_change};

struct ErrorRecover;

impl AfterMiddleware for ErrorRecover {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        match err.response.status {
            // each error will result in 404 response
            Some(_) => Ok(err.response.set(status::NotFound)),
            _ => Err(err),
        }
    }
}

// Watch command implementation
pub fn serve(args: &ArgMatches) -> Result<(), Box<Error>> {
    const RELOAD_COMMAND: &'static str = "reload";

    let book_dir = get_book_dir(args);
    let book = MDBook::new(&book_dir).read_config()?;

    let mut book = match args.value_of("dest-dir") {
        Some(dest_dir) => book.with_destination(Path::new(dest_dir)),
        None => book,
    };

    if let None = book.get_destination() {
        println!("The HTML renderer is not set up, impossible to serve the files.");
        std::process::exit(2);
    }

    if args.is_present("curly-quotes") {
        book = book.with_curly_quotes(true);
    }

    let port = args.value_of("port").unwrap_or("3000");
    let ws_port = args.value_of("websocket-port").unwrap_or("3001");
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
"#,
                                public_address,
                                ws_port,
                                RELOAD_COMMAND));

    book.build()?;

    let mut chain = Chain::new(staticfile::Static::new(book.get_destination()
                                                           .expect("destination is present, checked before")));
    chain.link_after(ErrorRecover);
    let _iron = Iron::new(chain).http(&*address).unwrap();

    let ws_server = ws::WebSocket::new(|_| |_| Ok(())).unwrap();

    let broadcaster = ws_server.broadcaster();

    std::thread::spawn(move || { ws_server.listen(&*ws_address).unwrap(); });

    let serving_url = format!("http://{}", address);
    println!("\nServing on: {}", serving_url);

    if open_browser {
        open(serving_url);
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
