// Package serve hosts the HTTP server used by `mdbook serve`. It is a
// port of the axum-based server in src/cmd/serve.rs. Two endpoints are
// served from the build directory:
//
//   - /<LiveReloadEndpoint>  WebSocket; sends a "reload" message on every
//                            build, so the page can refresh itself.
//   - <everything else>      static file from the build directory, with a
//                            404 fallback to <build>/<404 file>.
//
// The HTTP and reload layers are split into two files: serve.go (this
// file) hosts the listener and static file handler; reload.go implements
// the WebSocket broadcaster.
//
// Note: the static handler is hand-rolled rather than using
// http.FileServer, because Go stdlib's FileServer hard-codes a 301
// redirect for any URL ending in `/index.html`. That conflicts with
// mdBook's chapter URLs (which may literally end in `index.html`), and
// the Rust port does not exhibit the issue. See staticHandler below.
package serve

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
	"path"
	"path/filepath"
	"strconv"
	"strings"
)

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
