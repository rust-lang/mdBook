package driver

import (
	"context"
	"fmt"
	"os"
	"time"
)

// WatcherKind selects which filesystem-watching strategy to use. It maps
// directly onto src/cmd/watch.rs::WatcherKind in the Rust CLI:
//
//	"poll"    — walkdir + mtime/size on a 1s timer; works everywhere
//	"native"  — fsnotify with debounce; faster on local filesystems
type WatcherKind string

const (
	WatcherPoll   WatcherKind = "poll"
	WatcherNative WatcherKind = "native"
)

// ParseWatcherKind accepts both the kebab-case CLI value ("poll" /
// "native") and an empty string (which defaults to native). Unknown
// values return an error so the CLI can fail loudly rather than silently
// picking a backend.
func ParseWatcherKind(s string) (WatcherKind, error) {
	switch s {
	case "", "native":
		return WatcherNative, nil
	case "poll":
		return WatcherPoll, nil
	}
	return "", fmt.Errorf("unsupported watcher %q (expected 'poll' or 'native')", s)
}

// WatchOptions configures a Watch() invocation. UpdateConfig is called on
// every reload so callers can re-apply CLI flags (e.g. --dest-dir). It
// runs after the book has been reloaded from disk, so any config the
// user wrote to book.toml in the meantime is visible.
//
// PostBuild is invoked after a successful rebuild; serve uses it to
// broadcast a reload message over its WebSocket.
type WatchOptions struct {
	Kind         WatcherKind
	UpdateConfig func(*MDBook)
	PostBuild    func()
	// Debounce overrides the default 1s window. Zero means "use the
	// default"; non-zero is clamped to [100ms, 10s].
	Debounce time.Duration
}

// Watch blocks until ctx is cancelled, rebuilding the book every time a
// watched file changes. The first build runs synchronously before
// entering the wait loop so users see a populated output directory
// before the watcher starts idling.
//
// On any rebuild error the loop continues — the watcher should not die
// because of a transient TOML syntax error or a stale chapter. The error
// is logged via PostBuild's caller (typically the CLI's stderr writer).
func Watch(ctx context.Context, dir string, opts WatchOptions) error {
	if opts.Kind == "" {
		opts.Kind = WatcherNative
	}
	if opts.UpdateConfig == nil {
		opts.UpdateConfig = func(*MDBook) {}
	}
	if opts.PostBuild == nil {
		opts.PostBuild = func() {}
	}

	m, err := Load(dir)
	if err != nil {
		return fmt.Errorf("watch: load: %w", err)
	}
	opts.UpdateConfig(m)
	if err := m.Build(); err != nil {
		// Don't bail — the first build may legitimately fail (e.g. the
		// user is in the middle of editing book.toml). The next change
		// will retry.
		fmt.Fprintf(os.Stderr, "watch: initial build failed: %v\n", err)
	} else {
		opts.PostBuild()
	}

	roots := collectWatchRoots(m)

	switch opts.Kind {
	case WatcherPoll:
		return watchPoll(ctx, m, roots, opts)
	case WatcherNative:
		return watchNative(ctx, m, roots, opts)
	}
	return fmt.Errorf("watch: unknown kind %q", opts.Kind)
}

func watchPoll(ctx context.Context, m *MDBook, roots []string, opts WatchOptions) error {
	pw := NewPollWatcher(m.Root)
	pw.SetRoots(roots)
	// Prime the cache so the first tick doesn't flag every file as new.
	_ = pw.Scan()
	ticker := time.NewTicker(PollTick)
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

func watchNative(ctx context.Context, m *MDBook, roots []string, opts WatchOptions) error {
	nw, err := NewNativeWatcher(m.Root)
	if err != nil {
		return fmt.Errorf("watch: native: %w", err)
	}
	defer nw.Close()
	for _, r := range roots {
		if r == "" {
			continue
		}
		if _, err := os.Stat(r); err != nil {
			// Missing paths (e.g. theme/ that doesn't exist) are not a
			// fatal error — the rebuild will recreate them. Skip the
			// Add and continue.
			continue
		}
		if err := nw.Add(r); err != nil {
			return fmt.Errorf("watch: native add %s: %w", r, err)
		}
	}
	return nw.Run(ctx, func(paths []string) {
		if !rebuild(m.Root, opts) {
			return
		}
		opts.PostBuild()
	})
}

// rebuild reloads the book from disk, re-applies the CLI-level config
// override, and runs a fresh build. The boolean return reports whether
// the build succeeded so the caller knows whether to call PostBuild.
func rebuild(dir string, opts WatchOptions) bool {
	m, err := Load(dir)
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
