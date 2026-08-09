package html

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// goldenRoot points at the Rust test suite, whose expected/*.html files are the
// exact chapter bodies the Rust renderer produces. They are the oracle for this
// port.
const goldenRoot = "../../../tests/testsuite/markdown"

// defaultOptions mirrors the Rust defaults for [output.html] — the
// configuration those fixtures are rendered with.
func defaultOptions(path string) Options {
	return Options{
		Path:             path,
		SmartPunctuation: true,
		DefinitionLists:  true,
		Admonitions:      true,
	}
}

// knownDeviations lists golden files this port does not yet reproduce, with
// the reason. Each one is a parser-level difference between pulldown-cmark and
// goldmark rather than a bug in the tree building, so closing them means
// replacing part of goldmark's block parsing.
var knownDeviations = map[string]string{
	// goldmark's definition list requires a plain single-line term, so a term
	// that contains a link (or spans lines) stays a paragraph instead of
	// becoming <dt>.
	"definition_lists/definition_lists": "goldmark rejects terms containing inline markup",
	// An opening tag split across two lines is HTML block type 7 in goldmark
	// but falls back to a paragraph with inline HTML in pulldown-cmark.
	"basic_markdown/html": "multi-line open tag is treated as an HTML block",
	// The Rust oracle renders rust blocks with the last line glued to
	// </code></pre> — a side effect of its rust hide-lines pass rebuilding
	// the code children without a trailing newline. The Go port removed that
	// pass (Rust leftovers cleanup), so rust blocks now keep their trailing
	// newline like every other language.
	"basic_markdown/code-blocks": "rust hide-lines pass removed; rust blocks keep the trailing newline",
	"basic_markdown/blockquotes": "rust hide-lines pass removed; rust blocks keep the trailing newline",
}

func TestMarkdownGolden(t *testing.T) {
	dirs, err := os.ReadDir(goldenRoot)
	if err != nil {
		t.Skipf("Rust test suite not available: %v", err)
	}
	ran := 0
	for _, dir := range dirs {
		if !dir.IsDir() {
			continue
		}
		expectedDir := filepath.Join(goldenRoot, dir.Name(), "expected")
		expected, err := os.ReadDir(expectedDir)
		if err != nil {
			continue
		}
		for _, want := range expected {
			name := strings.TrimSuffix(want.Name(), ".html")
			srcPath := filepath.Join(goldenRoot, dir.Name(), "src", name+".md")
			source, err := os.ReadFile(srcPath)
			if err != nil {
				continue
			}
			wantHTML, err := os.ReadFile(filepath.Join(expectedDir, want.Name()))
			if err != nil {
				t.Fatal(err)
			}
			ran++
			t.Run(dir.Name()+"/"+name, func(t *testing.T) {
				if reason, known := knownDeviations[dir.Name()+"/"+name]; known {
					t.Skip("known deviation: " + reason)
				}
				got, err := Render(string(source), defaultOptions(name+".md"))
				if err != nil {
					t.Fatal(err)
				}
				// The golden files have no trailing newline; the renderer's
				// pretty printer emits one after a trailing block element.
				gotTrim := strings.TrimRight(got, "\n")
				wantTrim := strings.TrimRight(string(wantHTML), "\n")
				if gotTrim != wantTrim {
					reportLineDiff(t, gotTrim, wantTrim)
				}
			})
		}
	}
	if ran == 0 {
		t.Skip("no golden files found")
	}
}

// reportLineDiff prints the first few differing lines rather than both whole
// documents, which keeps failures readable while iterating.
func reportLineDiff(t *testing.T, got, want string) {
	t.Helper()
	gotLines, wantLines := strings.Split(got, "\n"), strings.Split(want, "\n")
	shown := 0
	for i := 0; i < len(gotLines) || i < len(wantLines); i++ {
		g, w := "<eof>", "<eof>"
		if i < len(gotLines) {
			g = gotLines[i]
		}
		if i < len(wantLines) {
			w = wantLines[i]
		}
		if g == w {
			continue
		}
		t.Errorf("line %d:\n got: %s\nwant: %s", i+1, g, w)
		if shown++; shown == 3 {
			return
		}
	}
}
