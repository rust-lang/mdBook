use super::command_prelude::*;
#[cfg(feature = "watch")]
use super::watch;
use crate::{get_book_dir, open};
use anyhow::Result;
use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::routing::get;
use clap::builder::NonEmptyStringValueParser;
use futures_util::StreamExt;
use futures_util::sink::SinkExt;
use mdbook_core::utils::fs::get_404_output_file;
use mdbook_driver::MDBook;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};

/// The HTTP endpoint for the websocket used to trigger reloads when a file changes.
const LIVE_RELOAD_ENDPOINT: &str = "__livereload";

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("serve")
        .about("Serves a book at http://localhost:3000, and rebuilds it on changes")
        .arg_dest_dir()
        .arg_root_dir()
        .arg(
            Arg::new("hostname")
                .short('n')
                .long("hostname")
                .num_args(1)
                .default_value("localhost")
                .value_parser(NonEmptyStringValueParser::new())
                .help("Hostname to listen on for HTTP connections"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .num_args(1)
                .default_value("3000")
                .value_parser(NonEmptyStringValueParser::new())
                .help("Port to use for HTTP connections"),
        )
        .arg_open()
        .arg_watcher()
}

// Serve command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(&book_dir)?;

    let port = args.get_one::<String>("port").unwrap();
    let hostname = args.get_one::<String>("hostname").unwrap();
    let open_browser = args.get_flag("open");

    let address = format!("{hostname}:{port}");

    let update_config = |book: &mut MDBook| {
        book.config
            .set("output.html.live-reload-endpoint", LIVE_RELOAD_ENDPOINT)
            .expect("live-reload-endpoint update failed");
        set_dest_dir(args, book);
        // Override site-url for local serving of the 404 file
        book.config.set("output.html.site-url", "/").unwrap();
    };
    update_config(&mut book);
    book.build()?;

    let sockaddr: SocketAddr = address
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("no address found for {}", address))?;
    let build_dir = book.build_dir_for("html");
    let html_config = book.config.html_config();
    let input_404 = html_config.and_then(|c| c.input_404);
    let file_404 = get_404_output_file(&input_404);

    // A channel used to broadcast to any websockets to reload when a file changes.
    let (tx, _rx) = tokio::sync::broadcast::channel::<Message>(100);

    let reload_tx = tx.clone();
    let thread_handle = std::thread::spawn(move || {
        serve(build_dir, sockaddr, reload_tx, &file_404);
    });

    let serving_url = format!("http://{address}");
    info!("Serving on: {}", serving_url);

    if open_browser {
        open(serving_url);
    }

    #[cfg(feature = "watch")]
    {
        let watcher = watch::WatcherKind::from_str(args.get_one::<String>("watcher").unwrap());
        watch::rebuild_on_change(watcher, &book_dir, &update_config, &move || {
            let _ = tx.send(Message::text("reload"));
        });
    }

    let _ = thread_handle.join();

    Ok(())
}

#[tokio::main]
async fn serve(
    build_dir: PathBuf,
    address: SocketAddr,
    reload_tx: broadcast::Sender<Message>,
    file_404: &str,
) {
    let reload_tx_clone = reload_tx.clone();

    // WebSocket handler for live reload
    let websocket_handler = move |ws: WebSocketUpgrade| async move {
        let reload_tx = reload_tx_clone.clone();
        ws.on_upgrade(move |socket| websocket_connection(socket, reload_tx))
    };

    let app = Router::new()
        .route(&format!("/{LIVE_RELOAD_ENDPOINT}"), get(websocket_handler))
        .fallback_service(
            ServeDir::new(&build_dir).not_found_service(ServeFile::new(build_dir.join(file_404))),
        );

    std::panic::set_hook(Box::new(move |panic_info| {
        // exit if serve panics
        error!("Unable to serve: {}", panic_info);
        std::process::exit(1);
    }));

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .unwrap_or_else(|e| panic!("Unable to bind to {address}: {e}"));

    axum::serve(listener, app).await.unwrap();
}

async fn websocket_connection(ws: WebSocket, reload_tx: broadcast::Sender<Message>) {
    let (mut user_ws_tx, _user_ws_rx) = ws.split();
    let mut rx = reload_tx.subscribe();

    trace!("websocket got connection");
    if let Ok(m) = rx.recv().await {
        trace!("notify of reload");
        let _ = user_ws_tx.send(m).await;
    }
}
