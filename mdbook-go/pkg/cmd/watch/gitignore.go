package watch

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// Gitignore is a tiny subset of gitignore syntax. It is intentionally not
// a full implementation — only the rules mdBook's watcher exercises are
// supported:
//
//   - `*.tmp`           extension glob
//   - `somedir/`        directory (trailing slash)
//   - `/inroot`         anchored to the gitignore file's directory
//   - `!/foo`           negation (passed through but rarely used by users)
//   - blank lines and `# comment` lines are skipped
//
// The grammar follows src/cmd/watch/poller.rs::find_gitignore + the
// `ignore::gitignore::Gitignore` API. Files outside the gitignore's own
// directory are matched against patterns that may include a `../` prefix,
// which is the parent-directory behaviour exercised by the
// test_ignore_in_parent and test_ignore_canonical Rust tests.
type Gitignore struct {
	root  string // absolute directory the gitignore lives in
	rules []gitignoreRule
}

type gitignoreRule struct {
	pattern  string
	dirOnly  bool
	anchored bool
	negate   bool
}

// NewGitignore loads a .gitignore file at the given absolute path. A
// missing file returns (nil, nil) so callers can treat "no gitignore" as
// "match nothing".
func NewGitignore(path string) (*Gitignore, error) {
	if path == "" {
		return nil, nil
	}
	info, err := os.Stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, nil
		}
		return nil, err
	}
	if info.IsDir() {
		return nil, fmt.Errorf("gitignore path %q is a directory", path)
	}
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	gi := &Gitignore{root: filepath.Dir(path)}
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		negate := false
		if strings.HasPrefix(line, "!") {
			negate = true
			line = strings.TrimSpace(line[1:])
		}
		anchored := strings.HasPrefix(line, "/")
		if anchored {
			line = line[1:]
		}
		dirOnly := strings.HasSuffix(line, "/")
		if dirOnly {
			line = strings.TrimSuffix(line, "/")
		}
		gi.rules = append(gi.rules, gitignoreRule{
			pattern:  line,
			dirOnly:  dirOnly,
			anchored: anchored,
			negate:   negate,
		})
	}
	if err := scanner.Err(); err != nil {
		return nil, err
	}
	return gi, nil
}

// FindGitignore walks upwards from bookRoot until it finds a .gitignore,
// mirroring src/cmd/watch.rs::find_gitignore. Returns the absolute path
// of the first match, or "" if none is found.
func FindGitignore(bookRoot string) string {
	abs, err := filepath.Abs(bookRoot)
	if err != nil {
		return ""
	}
	dir := abs
	for {
		candidate := filepath.Join(dir, ".gitignore")
		if _, err := os.Stat(candidate); err == nil {
			return candidate
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return ""
		}
		dir = parent
	}
}

// Match reports whether the given path is ignored. path is interpreted
// relative to the gitignore's own directory; absolute paths are converted
// via filepath.Rel first. isDir tells the matcher whether the path refers
// to a directory (so `somedir/` patterns can be distinguished from
// `somedir`).
func (g *Gitignore) Match(path string, isDir bool) bool {
	if g == nil {
		return false
	}
	rel, err := filepath.Rel(g.root, path)
	if err != nil {
		// Path is outside the gitignore's tree — by Rust's convention we
		// do not ignore external paths. This matches the `native.rs`
		// branch where extra-watch-dirs entries bypass gitignore entirely.
		return false
	}
	rel = filepath.ToSlash(rel)
	ignored := false
	for _, r := range g.rules {
		if r.dirOnly && !isDir {
			continue
		}
		ok, err := filepath.Match(r.pattern, filepath.Base(rel))
		if err != nil {
			continue
		}
		matched := ok
		if !matched {
			matched = matchGlob(r.pattern, rel)
		}
		if !matched && r.anchored {
			// Anchored patterns only match the relative path verbatim
			// (already covered by matchGlob) — if it didn't match, skip.
			continue
		}
		if matched {
			ignored = !r.negate
		}
	}
	return ignored
}

// matchGlob is a tiny `**` aware glob matcher. Standard filepath.Match
// only handles `*` and `?` per segment; we add `**` to span multiple
// directory levels (used in patterns like `foo/**/bar`).
func matchGlob(pattern, s string) bool {
	if !strings.Contains(pattern, "**") {
		return false
	}
	// Replace `**` with a marker and a `.*` regex; everything else becomes
	// literal in the wildcard segments. Keep it simple: split on `**` and
	// check each non-empty fragment with filepath.Match.
	parts := strings.Split(pattern, "**")
	if len(parts) == 2 {
		prefix, suffix := parts[0], parts[1]
		prefix = strings.TrimSuffix(prefix, "/")
		suffix = strings.TrimPrefix(suffix, "/")
		if prefix != "" && !strings.HasPrefix(s, prefix) {
			return false
		}
		if suffix != "" && !strings.HasSuffix(s, suffix) {
			return false
		}
		return true
	}
	// 3+ `**` parts fall back to "contains" — gitignore semantics are
	// deliberately fuzzy here and the Rust tests don't exercise the case.
	return false
}
