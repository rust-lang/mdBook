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
package serve

import (
	"context"
	"errors"
	"fmt"
	"log"
	"net"
	"net/http"
	"os"
	"path/filepath"
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
// file. This mirrors the axum `ServeDir::new(&build_dir).not_found_service
// (ServeFile::new(build_dir.join(file_404)))` chain in src/cmd/serve.rs.
func staticHandler(root, notFoundPath string, logger *log.Logger) http.Handler {
	fs := http.FileServer(http.Dir(root))
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Normalise the path and check existence; the net/http file
		// server would otherwise 404 with its own body, which differs
		// from the Rust build's 404.html content.
		upath := r.URL.Path
		if !strings.HasPrefix(upath, "/") {
			upath = "/" + upath
		}
		clean := filepath.FromSlash(filepath.Clean(upath))
		full := filepath.Join(root, clean)
		if info, err := os.Stat(full); err == nil && !info.IsDir() {
			fs.ServeHTTP(w, r)
			return
		}
		// Fallback: serve the configured 404 file with a 404 status.
		// The body is the file content; the status is what we set here.
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		w.WriteHeader(http.StatusNotFound)
		body, err := os.ReadFile(filepath.Join(root, notFoundPath))
		if err != nil {
			logger.Printf("serve: 404 fallback %s unreadable: %v", notFoundPath, err)
			_, _ = w.Write([]byte("Not Found"))
			return
		}
		_, _ = w.Write(body)
	})
}
