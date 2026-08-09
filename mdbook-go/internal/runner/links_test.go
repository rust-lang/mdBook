package runner

import (
	"strings"
	"testing"
)

// TestAnchorIncludeStripsOrphanAnchor covers the regression in
// tests/testsuite/includes/all_includes: a bare {{#include FILE:ANCHOR}}
// between two matching ANCHOR/ANCHOR_END markers nests an unrelated
// `// ANCHOR: unendinganchor` line. Rust's take_anchored_lines
// (crates/mdbook-driver/src/builtin_preprocessors/links/take_lines.rs:31-57)
// drops any line containing "ANCHOR:" once inside the block; before this
// fix the Go port leaked the orphan directive into the rendered HTML.
func TestAnchorIncludeStripsOrphanAnchor(t *testing.T) {
	src := `// preamble
// ANCHOR: myanchor
// ANCHOR: unendinganchor
let x = 1;
// ANCHOR_END: myanchor
// postamble
`
	r := lineRange{anchor: strPtr("myanchor")}
	got := r.apply(src)
	for _, banned := range []string{"ANCHOR: unendinganchor", "ANCHOR_END: myanchor"} {
		if strings.Contains(got, banned) {
			t.Errorf("anchor marker %q leaked into output:\n%s", banned, got)
		}
	}
	if !strings.Contains(got, "let x = 1;") {
		t.Errorf("expected inner content preserved, got:\n%s", got)
	}
}

func strPtr(s string) *string { return &s }
