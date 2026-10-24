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
use mdbook_driver::MDBook;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, info, trace};

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
                .value_parser(clap::value_parser!(u16))
                .help("Port to use for HTTP connections"),
        )
        .arg(
            Arg::new("socket-activate")
                .long("socket-activate")
                .num_args(0)
                .conflicts_with_all(["hostname", "port"])
                .help("Use a pre-bound socket from LISTEN_FDS (systemd/foreman socket activation)"),
        )
        .arg_open()
        .arg_watcher()
}

// Serve command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let mut book = MDBook::load(&book_dir)?;

    let port = *args.get_one::<u16>("port").unwrap();
    let hostname = args.get_one::<String>("hostname").unwrap();
    let open_browser = args.get_flag("open");
    let bind_explicitly_set = args.value_source("port")
        == Some(clap::parser::ValueSource::CommandLine)
        || args.value_source("hostname") == Some(clap::parser::ValueSource::CommandLine);
    let socket_activate = args.get_flag("socket-activate");

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

    // Two ways to obtain a listener; depending on the flags we try
    // one or both, in order.
    let from_env = || -> Option<std::net::TcpListener> {
        listenfd::ListenFd::from_env()
            .take_tcp_listener(0)
            .expect("failed to take listenfd TCP listener")
    };
    let from_bind = || -> Result<std::net::TcpListener> {
        let address = format!("{hostname}:{port}");
        let sockaddr: SocketAddr = address
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow::anyhow!("no address found for {}", address))?;
        Ok(std::net::TcpListener::bind(sockaddr)?)
    };

    let listener = if socket_activate {
        from_env().ok_or_else(|| {
            anyhow::anyhow!(
                "LISTEN_FDS not set or no TCP listener at fd 3; \
                 --socket-activate requires exactly one pre-bound TCP socket"
            )
        })?
    } else if bind_explicitly_set {
        from_bind()?
    } else {
        from_env().map_or_else(|| from_bind(), Ok)?
    };

    let local_addr = listener.local_addr()?;

    let build_dir = book.build_dir_for("html");
    let html_config = book.config.html_config().unwrap_or_default();
    let file_404 = html_config.get_404_output_file();

    // A channel used to broadcast to any websockets to reload when a file changes.
    let (tx, _rx) = tokio::sync::broadcast::channel::<Message>(100);

    let reload_tx = tx.clone();
    let thread_handle = std::thread::spawn(move || {
        serve(build_dir, listener, reload_tx, &file_404);
    });

    let serving_url = format!("http://{local_addr}");
    info!("Serving on: {serving_url}");

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
    std_listener: std::net::TcpListener,
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

    std_listener
        .set_nonblocking(true)
        .expect("failed to set nonblocking");
    let listener = tokio::net::TcpListener::from_std(std_listener)
        .expect("failed to convert listener to tokio");

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
