package watch

import (
	"os"
	"path/filepath"
	"sort"
	"sync"
	"time"

	"mdbook-go/internal/model"
	"mdbook-go/internal/runner"
)

// PollWatcher is a poll-based filesystem watcher. It walks the configured
// root paths once per tick (default 1 second) and emits the set of paths
// whose mtime or size changed since the previous scan. The implementation
// is a direct port of src/cmd/watch/poller.rs::Watcher.
//
// Poll is the only watcher backend in this port (the Rust CLI's
// fsnotify-based native backend is not ported): it works on any
// filesystem, including ones where the kernel does not reliably deliver
// change events (network mounts, some Docker bind mounts, certain WSL2
// configurations).
//
// PollWatcher is safe for concurrent Scan() calls but the typical use is
// a single goroutine that scans on a fixed timer.
type PollWatcher struct {
	rootPaths []string
	gitignore *Gitignore
	prev      map[string]pathStat
	mu        sync.Mutex
}

// pathStat is the cache key for a single file. Directories are skipped
// during scan (their mtime is unreliable on most filesystems) so only
// regular file entries live in the cache.
type pathStat struct {
	mtime time.Time
	size  int64
}

// mtime returns the file's modification time as reported by os.FileInfo.
// This mirrors src/cmd/watch/poller.rs::Watcher::scan, which uses
// `meta.modified().unwrap_or(SystemTime::UNIX_EPOCH)` directly with no
// platform-specific branch.
//
// Earlier revisions of this file tried to fall back to
// syscall.Stat_t.Ctim when ModTime() returned zero — a paranoid measure
// that did not actually compile on macOS (the field is named Ctimespec
// on Darwin, Ctim on Linux) and would have produced spurious rebuilds
// anyway, since ctime tracks metadata changes (chmod / rename / owner)
// rather than content. Reverting to the simpler "ModTime or zero"
// semantic keeps the Go watcher bit-for-bit equivalent to the Rust one.
func mtime(info os.FileInfo) time.Time {
	return info.ModTime()
}

// NewPollWatcher creates a watcher rooted at the given book directory.
// The gitignore (if any) is loaded once at construction time. Reloading
// the .gitignore when it changes is not implemented yet — the Rust
// version has a `// FIXME: ignore should be reloaded when it changes.`
// comment to match.
func NewPollWatcher(bookRoot string) *PollWatcher {
	gi, _ := NewGitignore(FindGitignore(bookRoot))
	return &PollWatcher{
		gitignore: gi,
		prev:      map[string]pathStat{},
	}
}

// SetRoots configures the directories the watcher will recursively scan.
// This is called from Watch() once the book has been loaded so the
// watcher's roots stay in sync with `[build] extra-watch-dirs` and the
// theme directory.
func (w *PollWatcher) SetRoots(roots []string) {
	w.mu.Lock()
	defer w.mu.Unlock()
	cleaned := make([]string, 0, len(roots))
	for _, r := range roots {
		if r == "" {
			continue
		}
		abs, err := filepath.Abs(r)
		if err != nil {
			continue
		}
		cleaned = append(cleaned, abs)
	}
	w.rootPaths = cleaned
}

// Scan walks the configured roots and returns the set of paths that
// changed since the previous scan. Both new paths and removed paths are
// reported; the caller is expected to treat every entry as "rebuild
// needed" without inspecting the diff.
func (w *PollWatcher) Scan() []string {
	w.mu.Lock()
	defer w.mu.Unlock()

	next := make(map[string]pathStat)
	for _, root := range w.rootPaths {
		if _, err := os.Stat(root); err != nil {
			continue
		}
		w.walkInto(root, next)
	}

	var changed []string
	for path, stat := range next {
		if old, ok := w.prev[path]; !ok || old != stat {
			changed = append(changed, path)
		}
	}
	for path := range w.prev {
		if _, ok := next[path]; !ok {
			changed = append(changed, path)
		}
	}
	w.prev = next
	sort.Strings(changed)
	return changed
}

// walkInto recursively walks root, applying the gitignore filter and
// recording regular-file stats. Symlinks are followed, matching the
// Rust `WalkDir::follow_links(true)` behaviour. The walker bails out
// of a directory as soon as a parent is gitignore-matched.
func (w *PollWatcher) walkInto(root string, into map[string]pathStat) {
	_ = filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			// A single unreadable entry should not abort the whole
			// scan; skip and continue. This matches the Rust
			// `filter_map` that drops entries whose metadata call
			// fails with a debug log.
			return nil
		}
		// Resolve symlinks so the cache key is stable across watches.
		if info.Mode()&os.ModeSymlink != 0 {
			resolved, err := filepath.EvalSymlinks(path)
			if err != nil {
				return nil
			}
			fi, err := os.Stat(resolved)
			if err != nil {
				return nil
			}
			info = fi
		}
		if info.IsDir() {
			isDir := true
			if w.gitignore != nil && w.gitignore.Match(path, isDir) {
				return filepath.SkipDir
			}
			return nil
		}
		if !info.Mode().IsRegular() {
			return nil
		}
		if w.gitignore != nil && w.gitignore.Match(path, false) {
			return nil
		}
		into[path] = pathStat{mtime: mtime(info), size: info.Size()}
		return nil
	})
}

// Tick is the default poll interval. The Rust implementation uses 1s;
// matching that keeps the user-facing experience consistent across
// implementations.
const PollTick = time.Second

// CollectWatchRoots computes the set of paths PollWatcher should scan.
// It mirrors src/cmd/watch/poller.rs::Watcher::set_roots: the source
// dir, the theme dir, the doclens.yaml, every entry in
// [build] extra-watch-dirs, and the additional-css / additional-js
// files declared in [output.html].
//
// HTML() can fail (e.g. malformed [output.html] table); on error we
// fall back to an empty additional-css/js list and let the render path
// surface the error later. The watcher itself doesn't need the config
// to be perfectly valid — it just needs enough to know what to scan.
func CollectWatchRoots(m *runner.MDBook) []string {
	roots := []string{
		m.SourceDir(),
		filepath.Join(m.Root, model.ConfigFileName),
	}
	if htmlCfg, err := m.Config.HTML(); err == nil {
		roots = append(roots, htmlCfg.ThemeDir(m.Root))
		for _, css := range htmlCfg.AdditionalCSS {
			roots = append(roots, filepath.Join(m.Root, css))
		}
		for _, js := range htmlCfg.AdditionalJS {
			roots = append(roots, filepath.Join(m.Root, js))
		}
	}
	for _, dir := range m.Config.Build.ExtraWatchDirs {
		roots = append(roots, filepath.Join(m.Root, dir))
	}
	return roots
}
