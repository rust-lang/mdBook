package plugin

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

// TestRustdocAnchorOutsideBlockPrefixed matches Rust's
// take_rustdoc_include_anchored_lines: lines OUTSIDE the matching anchor
// block are emitted with a `# ` prefix (so they render as
// <span class="boring">), and any ANCHOR/ANCHOR_END markers outside the
// block are dropped — they would otherwise surface as `# // ANCHOR: ...`
// boring spans. Byte-level parity is verified by the harness running
// tests/testsuite/includes/all_includes; the unit test here just asserts
// the structural invariants so future refactors don't regress them.
func TestRustdocAnchorOutsideBlockPrefixed(t *testing.T) {
	src := "fn some_other_function() {\n" +
		"    // ANCHOR: unused-anchor-that-should-be-stripped\n" +
		"    println!(\"unused anchor\");\n" +
		"    // ANCHOR_END: unused-anchor-that-should-be-stripped\n" +
		"}\n" +
		"\n" +
		"// ANCHOR: rustdoc-include-anchor\n" +
		"fn main() {\n" +
		"    some_other_function();\n" +
		"}\n" +
		"// ANCHOR_END: rustdoc-include-anchor\n"
	r := lineRange{anchor: strPtr("rustdoc-include-anchor")}
	got := r.applyRustdoc(src)

	// Invariants (rather than a byte-exact expectation which is brittle
	// to indentation differences in test source):
	for _, banned := range []string{"ANCHOR: unused", "ANCHOR_END: unused", "ANCHOR_END: rustdoc-include-anchor"} {
		if strings.Contains(got, banned) {
			t.Errorf("anchor marker %q must be dropped, got:\n%s", banned, got)
		}
	}
	if !strings.Contains(got, "fn main() {") {
		t.Errorf("expected visible block content, got:\n%s", got)
	}
	if !strings.Contains(got, "some_other_function()") {
		t.Errorf("expected inner block content, got:\n%s", got)
	}
	// Lines outside the block must carry the `# ` prefix.
	if !strings.Contains(got, "# fn some_other_function() {") {
		t.Errorf("expected outside-block line prefixed with `# `, got:\n%s", got)
	}
	if !strings.Contains(got, "#     println!") {
		t.Errorf("expected outside-block println! prefixed with `# `, got:\n%s", got)
	}
}

// TestRustdocAnchorTrailingNewline guards against the trailing-empty
// regression that previously emitted `<span class="boring"></span>`
// before </code></pre>. Rust's str::lines() drops a single trailing
// empty entry; splitLinesLikeRust mirrors that.
func TestRustdocAnchorTrailingNewline(t *testing.T) {
	src := "fn main() {}\n"
	r := lineRange{anchor: strPtr("doesntmatter")}
	got := r.applyRustdoc(src)
	if strings.Contains(got, "\n# \n") || strings.HasSuffix(got, "# \n") {
		t.Errorf("trailing empty leaked into rustdoc output:\n%q", got)
	}
	if !strings.Contains(got, "fn main() {}") {
		t.Errorf("expected visible content preserved, got:\n%q", got)
	}
}

func strPtr(s string) *string { return &s }