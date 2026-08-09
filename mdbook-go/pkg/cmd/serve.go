// The `doclens serve` subcommand: build the book, serve it over HTTP and
// live-reload on rebuild. Like Rust's src/cmd/serve.rs, the whole server
// lives in this one file: the HTTP listener and static file handler
// below, plus the WebSocket ReloadHub that broadcasts a "reload" message
// to connected pages after every rebuild. Two endpoints are served from
// the build directory:
//
//   - /<LiveReloadEndpoint>  WebSocket; sends a "reload" message on every
//     build, so the page can refresh itself.
//   - <everything else>      static file from the build directory, with a
//     404 fallback to <build>/<404 file>.
//
// Note: the static handler is hand-rolled rather than using
// http.FileServer, because Go stdlib's FileServer hard-codes a 301
// redirect for any URL ending in `/index.html`. That conflicts with
// mdBook's chapter URLs (which may literally end in `index.html`), and
// the Rust port does not exhibit the issue. See staticHandler below.
package cmd

import (
	"context"
	"errors"
	"fmt"
	"io"
	"log"
	"mime"
	"net"
	"net/http"
	"os"
	"os/signal"
	"path"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"

	"mdbook-go/internal/runner"

	"github.com/gorilla/websocket"
	"github.com/spf13/cobra"
)

// NewServeCommand implements the `doclens serve` subcommand.
func NewServeCommand() *cobra.Command {
	var dir, dest string
	var hostname, port string
	var openAfter bool

	cmd := &cobra.Command{
		Use:   "serve",
		Short: "serve the book + live reload",
		RunE: func(cmd *cobra.Command, args []string) error {
			return runServe(dir, dest, hostname, port, openAfter)
		},
	}

	cmd.Flags().StringVar(&dir, "dir", ".", "book root")
	cmd.Flags().StringVar(&dest, "dest-dir", "", "output directory (overrides doclens.yaml build-dir)")
	cmd.Flags().StringVar(&hostname, "hostname", "localhost", "hostname to listen on")
	cmd.Flags().StringVar(&port, "port", "3000", "TCP port for HTTP")
	cmd.Flags().BoolVar(&openAfter, "open", false, "open the served URL in the default browser after the listener is up")

	return cmd
}

// run is the entry point for `doclens serve`. It does the first
// build synchronously, starts the HTTP server, then runs the watcher in
// the foreground. Each successful rebuild calls Server.Reload() so the
// browser knows to refresh.
func runServe(dir, dest, hostname, port string, openAfter bool) error {
	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	m, err := runner.Load(dir)
	if err != nil {
		return err
	}
	if dest != "" {
		m.Config.Build.BuildDir = dest
	}
	// The Rust CLI overrides [output.html] live-reload-endpoint and
	// site-url for local serving. We set them on the same struct so
	// the rendered HTML points back at the same /__livereload path and
	// the 404 fallback resolves under "/".
	htmlCfg, err := m.Config.HTML()
	if err != nil {
		return err
	}
	htmlCfg.LiveReloadEndpoint = LiveReloadEndpoint
	htmlCfg.SiteURL = "/"
	if err := m.Build(); err != nil {
		return err
	}

	addr := net.JoinHostPort(hostname, port)
	srv := NewServer(Options{
		Addr:     addr,
		BuildDir: m.BuildDir(),
		NotFound: htmlCfg.Get404OutputFile(),
	})
	servingURL := "http://" + srv.Addr()

	if openAfter {
		// `Open` is fire-and-forget; we don't wait for the browser.
		_ = open(servingURL)
	}

	// Run the HTTP listener in its own goroutine so the watch loop can
	// drive rebuilds. The listener returns when ctx is cancelled.
	serveErr := make(chan error, 1)
	go func() { serveErr <- srv.Start(ctx) }()

	// Watcher runs in the foreground. Each rebuild that succeeds
	// triggers a Reload broadcast.
	watchErr := make(chan error, 1)
	go func() {
		watchErr <- Watch(ctx, dir, WatchOptions{
			UpdateConfig: func(b *runner.MDBook) {
				if dest != "" {
					b.Config.Build.BuildDir = dest
				}
				if hcfg, err := b.Config.HTML(); err == nil {
					hcfg.LiveReloadEndpoint = LiveReloadEndpoint
					hcfg.SiteURL = "/"
				}
			},
			PostBuild: func() { srv.Reload() },
		})
	}()

	select {
	case err := <-serveErr:
		cancel()
		return err
	case err := <-watchErr:
		cancel()
		return err
	}
}

// LiveReloadEndpoint is the path under which the WebSocket is served. The
// Rust CLI hard-codes the same value in src/cmd/serve.rs::LIVE_RELOAD_ENDPOINT;
// the book.js bundled in the default theme expects this exact path.
const LiveReloadEndpoint = "__livereload"

// Options configures a single Serve call.
type Options struct {
	// Addr is the listen address (e.g. "localhost:3000"). The empty
	// string is treated as "localhost:3000", matching clap's default.
	Addr string
	// BuildDir is the directory whose contents are served.
	BuildDir string
	// NotFound is the file to serve when no static file matches. The
	// default in the HTML backend is "404.html".
	NotFound string
	// Logger receives informational lines (startup, shutdown, errors).
	// nil falls back to log.Default().
	Logger *log.Logger
}

// Server wraps a net/http.Server plus a ReloadHub that broadcasts build
// notifications to connected WebSocket clients.
type Server struct {
	opts   Options
	http   *http.Server
	hub    *ReloadHub
	logger *log.Logger
}

// NewServer returns a Server bound to Options.BuildDir but not yet
// listening. Call Start to bind the listener; the returned channel emits
// the actual address (useful when Addr was ":0" and the OS picked a
// port).
func NewServer(opts Options) *Server {
	if opts.Addr == "" {
		opts.Addr = "localhost:3000"
	}
	if opts.NotFound == "" {
		opts.NotFound = "404.html"
	}
	if opts.Logger == nil {
		opts.Logger = log.Default()
	}
	hub := newReloadHub(opts.Logger)
	mux := http.NewServeMux()
	mux.HandleFunc("/"+LiveReloadEndpoint, hub.handle)
	mux.Handle("/", staticHandler(opts.BuildDir, opts.NotFound, opts.Logger))
	return &Server{
		opts:   opts,
		hub:    hub,
		logger: opts.Logger,
		http: &http.Server{
			Addr:    opts.Addr,
			Handler: mux,
		},
	}
}

// Start binds the listener. It blocks until the server stops. Cancel
// ctx to shut the server down gracefully; the listener will close and
// the function will return nil.
func (s *Server) Start(ctx context.Context) error {
	ln, err := net.Listen("tcp", s.opts.Addr)
	if err != nil {
		return fmt.Errorf("serve: listen %s: %w", s.opts.Addr, err)
	}
	s.logger.Printf("Serving on: http://%s", ln.Addr().String())

	// Bind the resolved address back so HTTP server picks it up. This is
	// the only way to honour an OS-assigned port while keeping
	// (*http.Server).Serve working — the field is read-only after
	// construction otherwise.
	s.http.Addr = ln.Addr().String()

	errCh := make(chan error, 1)
	go func() { errCh <- s.http.Serve(ln) }()

	select {
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithCancel(context.Background())
		defer cancel()
		_ = s.http.Shutdown(shutdownCtx)
		s.hub.close()
		return nil
	case err := <-errCh:
		if errors.Is(err, http.ErrServerClosed) {
			return nil
		}
		return err
	}
}

// Addr returns the actual listen address (only meaningful after Start
// has been called and bound the listener).
func (s *Server) Addr() string {
	if s.http == nil {
		return s.opts.Addr
	}
	return s.http.Addr
}

// Reload notifies all connected WebSocket clients that a rebuild
// finished. It is safe to call from any goroutine; the underlying hub
// uses a non-blocking send.
func (s *Server) Reload() {
	s.hub.broadcast()
}

// staticHandler returns an http.Handler that serves files from root and
// falls back to notFoundPath for any URL that does not match an existing
// file. This is hand-rolled (not http.FileServer) to match axum's
// `ServeDir::new(&build_dir).not_found_service(ServeFile::new(...))`
// behaviour used by src/cmd/serve.rs.
//
// Why hand-rolled: Go stdlib's http.FileServer calls serveFile, which
// hard-codes a 301 redirect for any URL ending in `/index.html`
// (see net/http/fs.go:679-688). That breaks mdBook, where the canonical
// URL of a chapter may literally be `index.html` (e.g. README.md →
// index.html). We bypass the redirect by going straight to os.Open +
// http.ServeContent, which streams the file without any redirect.
func staticHandler(root, notFoundPath string, logger *log.Logger) http.Handler {
	absRoot, err := filepath.Abs(root)
	if err != nil {
		// Fall back to the raw root; subsequent os.Open calls will fail
		// with a descriptive error if absRoot was needed for correctness.
		absRoot = root
	}
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		rel, ok := resolveStaticPath(r.URL.Path)
		if !ok {
			serveNotFound(w, absRoot, notFoundPath, logger)
			return
		}
		full := filepath.Join(absRoot, filepath.FromSlash(rel))
		info, err := os.Stat(full)
		if err != nil || info.IsDir() {
			serveNotFound(w, absRoot, notFoundPath, logger)
			return
		}
		f, err := os.Open(full)
		if err != nil {
			serveNotFound(w, absRoot, notFoundPath, logger)
			return
		}
		defer f.Close()
		if ctype := mime.TypeByExtension(filepath.Ext(full)); ctype != "" {
			w.Header().Set("Content-Type", ctype)
		}
		w.Header().Set("Content-Length", strconv.FormatInt(info.Size(), 10))
		w.Header().Set("Last-Modified", info.ModTime().UTC().Format(http.TimeFormat))
		w.WriteHeader(http.StatusOK)
		_, _ = io.Copy(w, f)
	})
}

// resolveStaticPath maps a URL path to a relative path inside the build
// directory. It mirrors the directory-handling rules of axum's ServeDir:
//   - "/"  → "index.html"
//   - "/foo/"  → "foo/index.html"
//   - "/foo.html" → "foo.html"
//   - "/foo/bar" → "foo/bar" (only served if it exists as a file)
//
// Returns ("", false) for malformed paths (those containing null bytes or
// that fail to clean). The caller treats false as 404.
func resolveStaticPath(upath string) (string, bool) {
	if !strings.HasPrefix(upath, "/") {
		upath = "/" + upath
	}
	// path.Clean always uses forward slash; URL paths are slash-separated.
	clean := path.Clean(upath)
	if clean == "/" {
		return "index.html", true
	}
	rel := strings.TrimPrefix(clean, "/")
	if rel == "" || strings.Contains(rel, "\x00") {
		return "", false
	}
	return rel, true
}

// serveNotFound writes the configured 404 file's body with a 404 status.
// On any failure (file missing, read error), falls back to a plain
// "Not Found" response so the client always gets a body.
func serveNotFound(w http.ResponseWriter, absRoot, notFoundPath string, logger *log.Logger) {
	body, err := os.ReadFile(filepath.Join(absRoot, filepath.FromSlash(notFoundPath)))
	if err != nil {
		if logger != nil {
			logger.Printf("serve: 404 fallback %s unreadable: %v", notFoundPath, err)
		}
		http.Error(w, "Not Found", http.StatusNotFound)
		return
	}
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.Header().Set("Content-Length", strconv.Itoa(len(body)))
	w.WriteHeader(http.StatusNotFound)
	_, _ = w.Write(body)
}

// reloadMessage is the literal payload the front-end book.js expects. The
// Rust implementation sends the same string over a tokio broadcast
// channel; we hard-code it here to keep the contract obvious. Changing
// it would require a matching change in the theme's book.js.
const reloadMessage = "reload"

// upgrader configures the WebSocket handshake. We allow any origin
// because the served book is expected to be opened via a local browser;
// if a user really wants to lock this down they can put mdbook behind
// a reverse proxy. The Rust CLI uses axum's default which is similarly
// permissive.
var upgrader = websocket.Upgrader{
	CheckOrigin:     func(r *http.Request) bool { return true },
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

// ReloadHub tracks active WebSocket clients and broadcasts a single
// "reload" message to all of them when a build finishes. It is the Go
// counterpart of the `tokio::sync::broadcast::channel::<Message>(100)`
// in src/cmd/serve.rs.
//
// The hub keeps one goroutine per client that pumps outgoing messages.
// Incoming messages are not consumed (the front-end doesn't send
// anything); the read loop exists only to detect closed connections.
type ReloadHub struct {
	mu      sync.Mutex
	clients map[*hubClient]struct{}
	logger  *log.Logger
	closed  bool
}

type hubClient struct {
	conn *websocket.Conn
	send chan string
}

// newReloadHub constructs a hub that logs to logger (or the default log).
func newReloadHub(logger *log.Logger) *ReloadHub {
	if logger == nil {
		logger = log.Default()
	}
	return &ReloadHub{
		clients: map[*hubClient]struct{}{},
		logger:  logger,
	}
}

// handle is the http.HandlerFunc mounted at /<LiveReloadEndpoint>. It
// upgrades the connection, registers the client, and starts its pump
// goroutine. The handler returns when the client disconnects.
func (h *ReloadHub) handle(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		// upgrader.Upgrade already wrote a response; we just log.
		h.logger.Printf("serve: ws upgrade: %v", err)
		return
	}
	client := &hubClient{conn: conn, send: make(chan string, 4)}
	h.add(client)
	defer h.remove(client)

	// Reader goroutine: discard everything, but exit when the conn
	// closes so the handler returns and we unregister.
	closed := make(chan struct{})
	go func() {
		defer close(closed)
		for {
			if _, _, err := conn.NextReader(); err != nil {
				return
			}
		}
	}()

	for {
		select {
		case <-closed:
			return
		case msg, ok := <-client.send:
			if !ok {
				return
			}
			// 5s write deadline so a wedged client can't pin the
			// hub goroutine forever.
			_ = conn.SetWriteDeadline(time.Now().Add(5 * time.Second))
			if err := conn.WriteMessage(websocket.TextMessage, []byte(msg)); err != nil {
				h.logger.Printf("serve: ws write: %v", err)
				return
			}
		}
	}
}

// broadcast enqueues a reload message on every connected client. The
// send is non-blocking — clients that are slow to drain their buffer
// simply miss this notification and will pick up the next one.
func (h *ReloadHub) broadcast() {
	h.mu.Lock()
	defer h.mu.Unlock()
	if h.closed {
		return
	}
	for c := range h.clients {
		select {
		case c.send <- reloadMessage:
		default:
			// Drop on the floor; the next broadcast will catch them.
		}
	}
}

func (h *ReloadHub) add(c *hubClient) {
	h.mu.Lock()
	defer h.mu.Unlock()
	h.clients[c] = struct{}{}
}

func (h *ReloadHub) remove(c *hubClient) {
	h.mu.Lock()
	defer h.mu.Unlock()
	if _, ok := h.clients[c]; !ok {
		return
	}
	delete(h.clients, c)
	close(c.send)
	_ = c.conn.Close()
}

func (h *ReloadHub) close() {
	h.mu.Lock()
	defer h.mu.Unlock()
	if h.closed {
		return
	}
	h.closed = true
	for c := range h.clients {
		_ = c.conn.Close()
		delete(h.clients, c)
	}
}
