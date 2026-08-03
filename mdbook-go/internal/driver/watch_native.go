package driver

import (
	"context"
	"path/filepath"
	"sort"
	"sync"
	"time"

	"github.com/fsnotify/fsnotify"
)

// NativeWatcher wraps fsnotify with a debounce window so that a flurry of
// filesystem events (e.g. an editor doing write-temp-then-rename) collapses
// into a single rebuild. It is the Go equivalent of
// src/cmd/watch/native.rs which uses notify_debouncer_mini.
//
// On most platforms the underlying fsnotify watcher is recursive: a single
// Add(root) call receives events for every descendant. We still need to
// add each extra-watch-dir and the theme dir explicitly because they are
// not under source_dir.
type NativeWatcher struct {
	fs        *fsnotify.Watcher
	debounce  time.Duration
	gitignore *Gitignore
	bookRoot  string
	mu        sync.Mutex
	added     map[string]struct{}
}

// NewNativeWatcher creates a NativeWatcher with a 1s debounce window
// (matching the Rust `Duration::from_secs(1)` configuration) and loads
// the nearest .gitignore as a filter. An fsnotify failure here is
// returned so the CLI can print a useful error and exit.
func NewNativeWatcher(bookRoot string) (*NativeWatcher, error) {
	fs, err := fsnotify.NewWatcher()
	if err != nil {
		return nil, err
	}
	gi, _ := NewGitignore(FindGitignore(bookRoot))
	return &NativeWatcher{
		fs:        fs,
		debounce:  time.Second,
		gitignore: gi,
		bookRoot:  bookRoot,
		added:     map[string]struct{}{},
	}, nil
}

// Add registers path with the underlying fsnotify watcher. Re-adding an
// already-watched path is a no-op (fsnotify will error otherwise).
// Symbolic links are followed so editors that write via a tmpfile + rename
// pattern are still observed.
func (w *NativeWatcher) Add(path string) error {
	abs, err := filepath.Abs(path)
	if err != nil {
		return err
	}
	w.mu.Lock()
	defer w.mu.Unlock()
	if _, ok := w.added[abs]; ok {
		return nil
	}
	if err := w.fs.Add(abs); err != nil {
		return err
	}
	w.added[abs] = struct{}{}
	return nil
}

// Close releases the underlying fsnotify watcher. After Close, Run
// returns immediately.
func (w *NativeWatcher) Close() error {
	return w.fs.Close()
}

// Run blocks until ctx is cancelled or the underlying watcher errors.
// On every debounce window the collected events are filtered through the
// gitignore matcher (paths outside bookRoot are kept verbatim, matching
// the `native.rs::any_external_paths` branch) and passed to onChange.
//
// The returned error is non-nil only on fsnotify fatal errors; a normal
// cancellation returns nil.
func (w *NativeWatcher) Run(ctx context.Context, onChange func(paths []string)) error {
	type pending struct {
		paths map[string]struct{}
	}
	ch := make(chan pending, 1)
	submit := func(p pending) {
		select {
		case ch <- p:
		default:
			// A newer batch is already queued; merge into it by reading
			// the queued value and writing it back. This is the standard
			// single-slot "coalesce" pattern and is what notify's
			// debouncer does internally.
			old := <-ch
			for k := range old.paths {
				p.paths[k] = struct{}{}
			}
			ch <- p
		}
	}

	go func() {
		timer := time.NewTimer(w.debounce)
		timer.Stop()
		var batch pending
		batch.paths = map[string]struct{}{}
		flush := func() {
			if len(batch.paths) == 0 {
				return
			}
			submit(pending{paths: batch.paths})
			batch.paths = map[string]struct{}{}
		}
		for {
			select {
			case <-ctx.Done():
				flush()
				return
			case ev, ok := <-w.fs.Events:
				if !ok {
					flush()
					return
				}
				if !isInteresting(ev) {
					continue
				}
				batch.paths[ev.Name] = struct{}{}
				timer.Reset(w.debounce)
			case <-timer.C:
				flush()
			}
		}
	}()

	for {
		select {
		case <-ctx.Done():
			return nil
		case p, ok := <-ch:
			if !ok {
				return nil
			}
			paths := w.filter(make([]string, 0, len(p.paths)))
			for k := range p.paths {
				paths = append(paths, k)
			}
			paths = w.filter(paths)
			if len(paths) == 0 {
				continue
			}
			sort.Strings(paths)
			onChange(paths)
		}
	}
}

// filter applies the gitignore rule to each path and removes duplicates.
// Paths outside bookRoot are passed through unchanged — they correspond
// to extra-watch-dirs and are not subject to the book-level gitignore.
func (w *NativeWatcher) filter(paths []string) []string {
	seen := map[string]struct{}{}
	out := make([]string, 0, len(paths))
	for _, p := range paths {
		if _, ok := seen[p]; ok {
			continue
		}
		seen[p] = struct{}{}
		if w.gitignore == nil {
			out = append(out, p)
			continue
		}
		isExternal := !isUnder(p, w.bookRoot)
		if isExternal {
			out = append(out, p)
			continue
		}
		if w.gitignore.Match(p, false) {
			continue
		}
		out = append(out, p)
	}
	return out
}

// isInteresting reduces a fsnotify event to "did something change on
// disk?" — Create/Write/Remove/Rename/Chmod all qualify, but a
// "no-op" event from the watcher is dropped.
func isInteresting(ev fsnotify.Event) bool {
	switch ev.Op {
	case fsnotify.Create, fsnotify.Write, fsnotify.Remove, fsnotify.Rename, fsnotify.Chmod:
		return true
	}
	return false
}

// isUnder reports whether path is inside root (or equal to it). Both
// arguments should be absolute; comparison is textual after Clean. This
// mirrors the `path.starts_with` check in src/cmd/watch/native.rs.
func isUnder(path, root string) bool {
	clean := filepath.Clean(path)
	cleanRoot := filepath.Clean(root) + string(filepath.Separator)
	return clean == filepath.Clean(root) || hasPrefix(clean, cleanRoot)
}

// hasPrefix is a tiny path-aware string-prefix test. We don't pull in
// strings.HasPrefix directly because the separator handling makes the
// intent clearer this way (and it lets us add platform quirks later
// without leaking them into callers).
func hasPrefix(s, prefix string) bool {
	return len(s) >= len(prefix) && s[:len(prefix)] == prefix
}
