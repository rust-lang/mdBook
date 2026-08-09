// The `doclens watch` subcommand and the filesystem-watching engine it
// shares with `serve`: rebuild the book whenever a source file changes.
// The command and the engine loop live here (mirroring mdbook's bin:
// `src/cmd/watch/mod.rs`), while the PollWatcher implementation lives in
// the pkg/cmd/watch subpackage (`poller.rs`); the native backend
// (native.rs) is not ported.
package cmd

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	"mdbook-go/internal/runner"
	"mdbook-go/pkg/cmd/watch"

	"github.com/spf13/cobra"
)

// NewWatchCommand implements the `doclens watch` subcommand.
func NewWatchCommand() *cobra.Command {
	var dir, dest string
	var openAfter bool

	cmd := &cobra.Command{
		Use:   "watch",
		Short: "rebuild on file changes",
		RunE: func(cmd *cobra.Command, args []string) error {
			return runWatch(dir, dest, openAfter)
		},
	}

	cmd.Flags().StringVar(&dir, "dir", ".", "book root")
	cmd.Flags().StringVar(&dest, "dest-dir", "", "output directory (overrides doclens.yaml build-dir)")
	cmd.Flags().BoolVar(&openAfter, "open", false, "open the rendered book in the default browser after the first build")

	return cmd
}

// run is the entry point for `doclens watch`. It installs a SIGINT
// handler so Ctrl-C exits cleanly and stops the poll loop.
func runWatch(dir, dest string, openAfter bool) error {
	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	if openAfter {
		// The Rust version does an initial build then opens the book in
		// the browser before entering the watch loop. We mirror that so
		// the user sees a populated book even if no further changes
		// occur.
		m, err := runner.Load(dir)
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
		if err := open(index); err != nil {
			return err
		}
	}

	return Watch(ctx, dir, WatchOptions{
		UpdateConfig: func(m *runner.MDBook) {
			if dest != "" {
				m.Config.Build.BuildDir = dest
			}
		},
	})
}

// WatchOptions configures a Watch() invocation. UpdateConfig is called on
// every reload so callers can re-apply CLI flags (e.g. --dest-dir). It
// runs after the book has been reloaded from disk, so any config the
// user wrote to doclens.yaml in the meantime is visible.
//
// PostBuild is invoked after a successful rebuild; serve uses it to
// broadcast a reload message over its WebSocket.
type WatchOptions struct {
	UpdateConfig func(*runner.MDBook)
	PostBuild    func()
}

// Watch blocks until ctx is cancelled, rebuilding the book every time a
// watched file changes. The first build runs synchronously before
// entering the wait loop so users see a populated output directory
// before the watcher starts idling.
//
// On any rebuild error the loop continues — the watcher should not die
// because of a transient doclens.yaml syntax error or a stale chapter. The
// error is logged to stderr and the next change retries the build.
//
// Only the poll backend (poller.go, a port of src/cmd/watch/poller.rs)
// is implemented; the Rust CLI's fsnotify-based native backend is not
// ported.
func Watch(ctx context.Context, dir string, opts WatchOptions) error {
	if opts.UpdateConfig == nil {
		opts.UpdateConfig = func(*runner.MDBook) {}
	}
	if opts.PostBuild == nil {
		opts.PostBuild = func() {}
	}

	m, err := runner.Load(dir)
	if err != nil {
		return fmt.Errorf("watch: load: %w", err)
	}
	opts.UpdateConfig(m)
	if err := m.Build(); err != nil {
		// Don't bail — the first build may legitimately fail (e.g. the
		// user is in the middle of editing doclens.yaml). The next change
		// will retry.
		fmt.Fprintf(os.Stderr, "watch: initial build failed: %v\n", err)
	} else {
		opts.PostBuild()
	}

	roots := watch.CollectWatchRoots(m)
	return watchPoll(ctx, m, roots, opts)
}

func watchPoll(ctx context.Context, m *runner.MDBook, roots []string, opts WatchOptions) error {
	pw := watch.NewPollWatcher(m.Root)
	pw.SetRoots(roots)
	// Prime the cache so the first tick doesn't flag every file as new.
	_ = pw.Scan()
	ticker := time.NewTicker(watch.PollTick)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			changed := pw.Scan()
			if len(changed) == 0 {
				continue
			}
			if !rebuild(m.Root, opts) {
				continue
			}
			opts.PostBuild()
		}
	}
}

// rebuild reloads the book from disk, re-applies the CLI-level config
// override, and runs a fresh build. The boolean return reports whether
// the build succeeded so the caller knows whether to call PostBuild.
func rebuild(dir string, opts WatchOptions) bool {
	m, err := runner.Load(dir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "watch: reload failed: %v\n", err)
		return false
	}
	opts.UpdateConfig(m)
	if err := m.Build(); err != nil {
		fmt.Fprintf(os.Stderr, "watch: build failed: %v\n", err)
		return false
	}
	return true
}
