// mdbook-go is the Go-language port of mdBook.
//
// Milestone coverage (see doc/plan/progress.md for the authoritative
// checklist):
//
//	M1 — core loader + minimal build           done
//	M2 — HTML renderer                          done (byte-identical to Rust on basic / nested)
//	M3 — preprocessor / renderer protocol       code done; external plugin e2e frozen
//	M4 — CLI surface (init / clean / test /    done (byte-identical to Rust on cli)
//	      build -open / watch / serve /         + completions / unified error handler)
//	      completions)
//	M5 — dev loop (watch poll + native, serve   done (byte-identical to Rust on serve)
//	      + WebSocket live reload)
//	M6 — cross-platform / CI / release /        not started
//	      large fixture library
//
// Note: the previous version string claimed M4+M5 e2e were blocked by a
// build memory regression; that regression has since been fixed and the
// cli + serve fixtures now pass strict-mode byte diff (2026-08-04). The
// 2026-08-04 session 8 also fixed a Linux-only stat.Ctim compile error
// in internal/driver/watch_poll_unix.go so the build now succeeds on
// macOS as well as Linux / Windows.
package main

import (
	"context"
	"flag"
	"fmt"
	"net"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"syscall"

	"mdbook-go/internal/cli"
	"mdbook-go/internal/completions"
	"mdbook-go/internal/driver"
	"mdbook-go/internal/serve"
)

func main() {
	if len(os.Args) < 2 {
		usage()
		os.Exit(cli.ExitCode)
	}
	cmd := os.Args[1]
	args := os.Args[2:]

	switch cmd {
	case "init":
		fs := flag.NewFlagSet("init", flag.ExitOnError)
		dir := fs.String("dir", ".", "book root")
		theme := fs.Bool("theme", false, "copy default theme into <dir>/theme")
		force := fs.Bool("force", false, "skip confirmation prompts (no prompts exist yet; accepted for parity with Rust)")
		title := fs.String("title", "", "book title (default \"My Book\" if empty)")
		ignore := fs.String("ignore", "git", "VCS ignore file to create: 'git' (default) or 'none'")
		_ = fs.Parse(args)
		cli.HandleError(driver.Init(*dir, driver.InitOptions{
			Title:  *title,
			Theme:  *theme,
			Force:  *force,
			Ignore: *ignore,
		}))
	case "build":
		fs := flag.NewFlagSet("build", flag.ExitOnError)
		dir := fs.String("dir", ".", "book root")
		dest := fs.String("dest-dir", "", "output directory (overrides book.toml)")
		open := fs.Bool("open", false, "open the rendered book in the default browser after building")
		_ = fs.Parse(args)
		cli.HandleError(runBuild(*dir, *dest, *open))
	case "clean":
		fs := flag.NewFlagSet("clean", flag.ExitOnError)
		dir := fs.String("dir", ".", "book root")
		dest := fs.String("dest-dir", "", "directory to remove (overrides book.toml build-dir)")
		_ = fs.Parse(args)
		cli.HandleError(runClean(*dir, *dest))
	case "test":
		fs := flag.NewFlagSet("test", flag.ExitOnError)
		dir := fs.String("dir", ".", "book root")
		chapter := fs.String("chapter", "", "only test the given chapter name or path")
		libraryPath := fs.String("library-path", "", "comma-separated directories to forward as -L to rustdoc")
		_ = fs.Parse(args)
		cli.HandleError(runTest(*dir, *chapter, *libraryPath))
	case "watch":
		fs := flag.NewFlagSet("watch", flag.ExitOnError)
		dir := fs.String("dir", ".", "book root")
		dest := fs.String("dest-dir", "", "output directory (overrides book.toml build-dir)")
		open := fs.Bool("open", false, "open the rendered book in the default browser after the first build")
		watcher := fs.String("watcher", "native", "filesystem watcher: 'native' (fsnotify + debounce) or 'poll' (walkdir + mtime/size)")
		_ = fs.Parse(args)
		cli.HandleError(runWatch(*dir, *dest, *open, *watcher))
	case "serve":
		fs := flag.NewFlagSet("serve", flag.ExitOnError)
		dir := fs.String("dir", ".", "book root")
		dest := fs.String("dest-dir", "", "output directory (overrides book.toml build-dir)")
		hostname := fs.String("hostname", "localhost", "hostname to listen on")
		port := fs.String("port", "3000", "TCP port for HTTP")
		open := fs.Bool("open", false, "open the served URL in the default browser after the listener is up")
		_ = fs.Parse(args)
		cli.HandleError(runServe(*dir, *dest, *hostname, *port, *open))
	case "completions":
		fs := flag.NewFlagSet("completions", flag.ExitOnError)
		shell := fs.String("shell", "", "target shell: bash|zsh|fish|powershell")
		_ = fs.Parse(args)
		// Allow the shell to be passed either as a flag or as the first
		// positional argument (so `mdbook-go completions bash` works the
		// same as the Rust invocation).
		if *shell == "" && fs.NArg() > 0 {
			*shell = fs.Arg(0)
		}
		sh, err := completions.ParseShell(*shell)
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(cli.ExitCode)
		}
		cli.HandleError(completions.Generate(os.Stdout, sh))
	case "version", "--version", "-v":
		fmt.Println("mdbook-go 0.1.0 (M1+M2 done; M3 frozen at 9/11; M4 + M5 e2e green: basic 40 / nested 48 / cli 37 / serve 38 byte-identical; M4.4 completions + M4.6/7 unified error handler landed)")
	default:
		usage()
		os.Exit(cli.ExitCode)
	}
}

func runBuild(dir, dest string, openAfter bool) error {
	m, err := driver.Load(dir)
	if err != nil {
		return err
	}
	if dest != "" {
		m.Config.Build.BuildDir = dest
	}
	if err := m.Build(); err != nil {
		return err
	}
	if openAfter {
		// The Rust version opens <build_dir>/html/index.html. Our default
		// build directory is the user's build-dir, where the index lives
		// directly as index.html (M2 outputs it alongside the per-chapter
		// pages, 404.html, etc.).
		index := filepath.Join(m.BuildDir(), "index.html")
		if err := driver.Open(index); err != nil {
			return err
		}
	}
	return nil
}

// runClean removes the book's build directory. It loads the book first to
// resolve the configured build-dir, but if dest is non-empty the override
// is honoured before Load is even called (matching Rust's `--dest-dir`
// semantics in src/cmd/clean.rs).
func runClean(dir, dest string) error {
	if dest != "" {
		// Override path: no need to load the book at all.
		c, err := driver.RemoveDir(dest)
		if err != nil {
			return err
		}
		fmt.Println(c)
		return nil
	}
	m, err := driver.Load(dir)
	if err != nil {
		return err
	}
	c, err := m.Clean("")
	if err != nil {
		return err
	}
	fmt.Println(c)
	return nil
}

// runWatch is the entry point for `mdbook watch`. It installs a SIGINT
// handler so Ctrl-C exits cleanly without leaving a dead goroutine on
// fsnotify's internal pipe.
func runWatch(dir, dest string, openAfter bool, watcherKind string) error {
	kind, err := driver.ParseWatcherKind(watcherKind)
	if err != nil {
		return err
	}
	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	if openAfter {
		// The Rust version does an initial build then opens the book in
		// the browser before entering the watch loop. We mirror that so
		// the user sees a populated book even if no further changes
		// occur.
		m, err := driver.Load(dir)
		if err != nil {
			return err
		}
		if dest != "" {
			m.Config.Build.BuildDir = dest
		}
		if err := m.Build(); err != nil {
			return err
		}
		index := filepath.Join(m.BuildDir(), "index.html")
		if err := driver.Open(index); err != nil {
			return err
		}
	}

	return driver.Watch(ctx, dir, driver.WatchOptions{
		Kind: kind,
		UpdateConfig: func(m *driver.MDBook) {
			if dest != "" {
				m.Config.Build.BuildDir = dest
			}
		},
	})
}

// runServe is the entry point for `mdbook serve`. It does the first
// build synchronously, starts the HTTP server, then runs the watcher in
// the foreground. Each successful rebuild calls Server.Reload() so the
// browser knows to refresh.
func runServe(dir, dest, hostname, port string, openAfter bool) error {
	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	m, err := driver.Load(dir)
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
	htmlCfg.LiveReloadEndpoint = serve.LiveReloadEndpoint
	htmlCfg.SiteURL = "/"
	if err := m.Build(); err != nil {
		return err
	}

	addr := net.JoinHostPort(hostname, port)
	srv := serve.NewServer(serve.Options{
		Addr:     addr,
		BuildDir: m.BuildDir(),
		NotFound: htmlCfg.Get404OutputFile(),
	})
	servingURL := "http://" + srv.Addr()

	if openAfter {
		// `Open` is fire-and-forget; we don't wait for the browser.
		_ = driver.Open(servingURL)
	}

	// Run the HTTP listener in its own goroutine so the watch loop can
	// drive rebuilds. The listener returns when ctx is cancelled.
	serveErr := make(chan error, 1)
	go func() { serveErr <- srv.Start(ctx) }()

	// Watcher runs in the foreground. Each rebuild that succeeds
	// triggers a Reload broadcast.
	watchErr := make(chan error, 1)
	go func() {
		watchErr <- driver.Watch(ctx, dir, driver.WatchOptions{
			Kind: driver.WatcherNative,
			UpdateConfig: func(b *driver.MDBook) {
				if dest != "" {
					b.Config.Build.BuildDir = dest
				}
				if hcfg, err := b.Config.HTML(); err == nil {
					hcfg.LiveReloadEndpoint = serve.LiveReloadEndpoint
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
// The libraryPath flag accepts a comma-separated list of directories which
// are forwarded to rustdoc as `-L <dir>`. Exit code 101 is returned when
// one or more chapters fail, matching Rust's "One or more tests failed".
func runTest(dir, chapter, libraryPath string) error {
	m, err := driver.Load(dir)
	if err != nil {
		return err
	}
	opts := driver.TestOptions{Chapter: chapter}
	if libraryPath != "" {
		// Match Rust's clap value_delimiter(','): trim whitespace around
		// each entry and drop empties so users can pass "a, b, ,c".
		for _, p := range strings.Split(libraryPath, ",") {
			if p = strings.TrimSpace(p); p != "" {
				opts.LibraryPaths = append(opts.LibraryPaths, p)
			}
		}
	}
	res, err := m.Test(opts)
	if err != nil {
		return err
	}
	if res.Failed > 0 {
		return fmt.Errorf("%d chapter(s) failed rustdoc tests", res.Failed)
	}
	return nil
}

func usage() {
	fmt.Fprintln(os.Stderr, "mdbook-go <command> [args]")
	fmt.Fprintln(os.Stderr, "  init         [-dir DIR] [-theme] [-force] [-title T] [-ignore git|none]")
	fmt.Fprintln(os.Stderr, "                                                             create a new book skeleton")
	fmt.Fprintln(os.Stderr, "  build        [-dir DIR] [-dest-dir DIR] [-open]           build a book")
	fmt.Fprintln(os.Stderr, "  clean        [-dir DIR] [-dest-dir DIR]                   remove the build directory")
	fmt.Fprintln(os.Stderr, "  test         [-dir DIR] [-chapter NAME] [-library-path DIR]")
	fmt.Fprintln(os.Stderr, "                                                             run rustdoc --test on chapters")
	fmt.Fprintln(os.Stderr, "  watch        [-dir DIR] [-dest-dir DIR] [-open] [-watcher poll|native]")
	fmt.Fprintln(os.Stderr, "                                                             rebuild on file changes")
	fmt.Fprintln(os.Stderr, "  serve        [-dir DIR] [-dest-dir DIR] [-hostname H] [-port P] [-open]")
	fmt.Fprintln(os.Stderr, "                                                             serve the book + live reload")
	fmt.Fprintln(os.Stderr, "  completions  [-shell bash|zsh|fish|powershell]            print shell completion script")
	fmt.Fprintln(os.Stderr, "  version                                                  show version")
}
