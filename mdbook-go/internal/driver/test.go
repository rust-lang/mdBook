package driver

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"

	"mdbook-go/internal/book"
	"mdbook-go/internal/plugin"
)

// TestOptions configures a single invocation of MDBook.Test. It mirrors the
// flags exposed by `mdbook test` in crates/mdbook/src/cmd/test.rs:
//
//	--chapter <name>      restrict to one chapter (name or path)
//	-L / --library-path   directories to forward as -L <dir> to rustdoc
//	colors                forward --color always when stderr is a TTY
type TestOptions struct {
	Chapter      string
	LibraryPaths []string
}

// TestResult captures the outcome of a test run so the CLI can render a
// summary line and pick a non-zero exit code on failure.
type TestResult struct {
	ChaptersRun int
	Failed      int
}

// Test runs `rustdoc --test` against every chapter in the book. The
// preprocessor chain runs first, so the same code blocks users see in the
// rendered output are the ones being tested. Each chapter is materialised
// in a fresh temp directory because rustdoc expects a real filesystem path.
//
// Behaviour matches crates/mdbook-driver/src/mdbook.rs::test_chapter:
//
//   - The book is preprocessed once; `links`/`index`/user preprocessors all
//     run as they would for a normal build.
//   - Each non-draft chapter is written to <tmp>/<chapter.path>.
//   - `rustdoc <chapter.path> --test` is invoked with the book's edition
//     and any --library-path entries passed through as `-L <path>`.
//   - On failure, the chapter's rustdoc stdout+stderr are echoed to our
//     stderr and the run continues with the next chapter.
func (m *MDBook) Test(opts TestOptions) (*TestResult, error) {
	pre, err := plugin.BuildPreprocessors(m.Config, m.Root)
	if err != nil {
		return nil, err
	}
	processed, _, err := plugin.RunPreprocessors(pre, m.Config, m.Root, "test", m.Book)
	if err != nil {
		return nil, err
	}

	tmpDir, err := os.MkdirTemp("", "mdbook-")
	if err != nil {
		return nil, fmt.Errorf("create temp dir: %w", err)
	}
	// Best-effort cleanup. The OS reaps the directory either way when the
	// process exits, but cleaning here keeps CI workspaces tidy.
	defer os.RemoveAll(tmpDir)

	if _, err := exec.LookPath("rustdoc"); err != nil {
		return nil, fmt.Errorf("`rustdoc` not found in PATH; install Rust to use `mdbook test`")
	}

	libraryArgs := buildLibraryArgs(opts.LibraryPaths)
	edition := m.Config.Rust.Edition
	color := isTerminal(os.Stderr)

	var res TestResult
	chapterFound := false
	processed.Iter(func(ch *book.Chapter) bool {
		if ch.IsDraft() {
			return true
		}
		if ch.Path == "" {
			return true
		}
		if opts.Chapter != "" && ch.Name != opts.Chapter && ch.Path != opts.Chapter {
			return true
		}
		chapterFound = true
		res.ChaptersRun++

		if err := writeChapter(tmpDir, ch); err != nil {
			fmt.Fprintf(os.Stderr, "ERROR writing chapter %s: %v\n", ch.Path, err)
			res.Failed++
			return true
		}

		args := []string{ch.Path, "--test"}
		args = append(args, libraryArgs...)
		if edition != "" {
			args = append(args, "--edition", edition)
		}
		if color {
			args = append(args, "--color", "always")
		}

		cmd := exec.Command("rustdoc", args...)
		cmd.Dir = tmpDir
		// rustdoc writes compile/test output to stderr; we forward it
		// directly so users see progress in real time.
		cmd.Stderr = os.Stderr
		cmd.Stdout = io.Discard
		if err := cmd.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "ERROR rustdoc returned an error for chapter %q: %v\n", ch.Name, err)
			res.Failed++
		}
		return true
	})

	if opts.Chapter != "" && !chapterFound {
		return &res, fmt.Errorf("chapter %q not found", opts.Chapter)
	}
	return &res, nil
}

// writeChapter dumps ch.Content to <tmpDir>/<ch.Path>, creating any
// intermediate directories. rustdoc requires the source file to exist on
// disk so it can derive a module name from the path.
func writeChapter(tmpDir string, ch *book.Chapter) error {
	full := filepath.Join(tmpDir, filepath.FromSlash(ch.Path))
	if err := os.MkdirAll(filepath.Dir(full), 0o755); err != nil {
		return err
	}
	return os.WriteFile(full, []byte(ch.Content), 0o644)
}

// buildLibraryArgs flattens --library-path entries into the `-L <path>`
// pairs that rustdoc accepts. Relative paths are resolved against the
// current working directory before being passed across the exec boundary,
// matching the Rust implementation in mdbook.rs::test_chapter.
func buildLibraryArgs(paths []string) []string {
	if len(paths) == 0 {
		return nil
	}
	cwd, err := os.Getwd()
	if err != nil {
		cwd = "."
	}
	out := make([]string, 0, len(paths)*2)
	for _, p := range paths {
		abs := p
		if !filepath.IsAbs(p) {
			abs = filepath.Join(cwd, p)
		}
		out = append(out, "-L", abs)
	}
	return out
}

// isTerminal is a minimal "is this file a TTY?" check. We don't import
// golang.org/x/term to avoid a new dependency for a single call site; the
// common case (CI / piped output) reports false and the Rust code does
// the same heuristic via is_terminal().
func isTerminal(f *os.File) bool {
	info, err := f.Stat()
	if err != nil {
		return false
	}
	return (info.Mode() & os.ModeCharDevice) != 0
}
