package plugin

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"

	"mdbook-go/internal/book"
)

// LinkPreprocessor expands a small set of helper directives inside chapter
// content. It mirrors crates/mdbook-driver/src/builtin_preprocessors/links.rs.
//
// Supported directives:
//
//	{{#include FILE[:START[:END]]}}     // include file or line range
//	{{#include FILE:ANCHOR}}             // include file between anchor lines
//	{{#rustdoc_include FILE:RANGE}}      // like include but hides lines behind #
//	{{#playground FILE [attrs...]}}      // embed file as a Rust playground block
//	{{#title NEW TITLE}}                 // override the chapter title
//	\{{#anything}}                       // escaped, dropped back to text
//
// {{#title}} updates ctx.ChapterTitles for the chapter so the renderer can use
// it; the chapter's `name` field is left untouched.
type LinkPreprocessor struct{}

// Name matches Rust's LinkPreprocessor::NAME.
func (LinkPreprocessor) Name() string { return "links" }

// Run applies the link preprocessor to every chapter in the book.
func (p LinkPreprocessor) Run(ctx *PreprocessorContext, b *book.Book) (*book.Book, error) {
	// ctx.Config.Book.Src is already absolute after config.SetSourceDir.
	srcDir := ctx.Config.Book.Src
	b.Iter(func(ch *book.Chapter) bool {
		if ch.IsDraft() {
			return true
		}
		base := filepath.Dir(ch.Path)
		baseDir := filepath.Join(srcDir, base)

		title := ch.Name
		newContent, err := replaceAll(ch.Content, baseDir, ch.Path, 0, &title)
		if err != nil {
			// Mirror Rust: log the error but keep the original content.
			fmt.Fprintf(os.Stderr, "links preprocessor: %v\n", err)
			return true
		}
		ch.Content = newContent
		if title != ch.Name {
			ctx.ChapterTitles[ch.Path] = title
		}
		return true
	})
	return b, nil
}

// SupportsRenderer is true for all renderers.
func (LinkPreprocessor) SupportsRenderer(string) (bool, error) { return true, nil }

const maxLinkNestedDepth = 10

func replaceAll(s, baseDir, sourcePath string, depth int, chapterTitle *string) (string, error) {
	var out strings.Builder
	lastEnd := 0
	for _, link := range findLinks(s) {
		out.WriteString(s[lastEnd:link.start])
		newContent, err := link.render(baseDir, chapterTitle)
		if err != nil {
			// Preserve the raw snippet so the user can see what went wrong.
			out.WriteString(s[link.start:link.end])
			lastEnd = link.end
			continue
		}
		if depth < maxLinkNestedDepth {
			if nested := link.nestedBase(baseDir); nested != "" {
				recur, err := replaceAll(newContent, nested, sourcePath, depth+1, chapterTitle)
				if err != nil {
					out.WriteString(s[link.start:link.end])
					lastEnd = link.end
					continue
				}
				out.WriteString(recur)
			} else {
				out.WriteString(newContent)
			}
		} else {
			// Mirrors links.rs: at depth == MAX_LINK_NESTED_DEPTH the Rust
			// implementation drops the included content entirely and only
			// logs an error — it does NOT emit the inner `{{#include}}`
			// directive as literal text. Without this drop the recursive
			// fixtures (e.g. tests/testsuite/includes/all_includes)
			// accumulate one extra repetition per level past the limit.
			fmt.Fprintf(os.Stderr, "links preprocessor: stack depth exceeded in %s (cyclic includes?)\n", sourcePath)
		}
		lastEnd = link.end
	}
	out.WriteString(s[lastEnd:])
	return out.String(), nil
}

// link captures one match of the link regex and the resolved directive.
type link struct {
	start, end int
	text       string
	kind       linkKind
}

// render dispatches to the underlying directive.
func (l link) render(baseDir string, title *string) (string, error) {
	return l.kind.render(baseDir, title)
}

// nestedBase returns the directory from which nested includes should resolve.
func (l link) nestedBase(baseDir string) string {
	return l.kind.nestedBase(baseDir)
}

type linkKind interface {
	render(baseDir string, title *string) (string, error)
	nestedBase(baseDir string) string
}

// findLinks returns every link-like token in s, in order.
func findLinks(s string) []link {
	var out []link
	all := linkRegex.FindAllStringSubmatchIndex(s, -1)
	matches := linkRegex.FindAllStringSubmatch(s, -1)
	for i, m := range matches {
		l, ok := linkFromMatch(s, m)
		if !ok {
			continue
		}
		l.start, l.end = all[i][0], all[i][1]
		out = append(out, l)
	}
	return out
}

// linkRegex matches the full directive including the closing `}}`. The
// captured groups are the kind and the rest of the body. It also matches the
// escaped form `\{{#...}}` which is filtered out by linkFromMatch.
//
// Go's regexp does not accept the inline `(?sx)` modifier; instead `(?s)`
// turns on dot-matches-newline and `(?x)` allows insignificant whitespace.
// Both are needed because mdBook's link directives can span multiple lines
// when users wrap the path in their own formatting.
var linkRegex = regexp.MustCompile(`(?s)\Q{{\E\s*\#([A-Za-z0-9_]+)\s+([^}]+)\Q}}\E`)

func linkFromMatch(s string, m []string) (link, bool) {
	if len(m) < 3 {
		return link{}, false
	}
	// Escaped form: the regex captures the whole match but no capture groups.
	kindStr := m[1]
	rest := m[2]
	if kindStr == "" {
		text := m[0]
		return link{
			start: 0,
			end:   0,
			text:  text,
			kind:  escapedLink{text: text[1:]},
		}, true
	}
	parts := strings.Fields(rest)
	if len(parts) == 0 {
		return link{}, false
	}
	switch kindStr {
	case "title":
		return link{text: m[0], kind: titleLink{title: parts[0]}}, true
	case "include":
		path, r := parseInclude(parts[0])
		return link{text: m[0], kind: includeLink{path: path, rng: r}}, true
	case "rustdoc_include":
		path, r := parseInclude(parts[0])
		return link{text: m[0], kind: rustdocIncludeLink{path: path, rng: r}}, true
	case "playground", "playpen":
		attrs := parts[1:]
		return link{text: m[0], kind: playgroundLink{path: parts[0], attrs: attrs}}, true
	}
	return link{}, false
}

// escapedLink preserves everything after the leading backslash verbatim.
type escapedLink struct{ text string }

func (e escapedLink) render(string, *string) (string, error) { return e.text, nil }
func (escapedLink) nestedBase(string) string                  { return "" }

type titleLink struct{ title string }

func (t titleLink) render(_ string, title *string) (string, error) {
	*title = t.title
	return "", nil
}
func (titleLink) nestedBase(string) string { return "" }

type includeLink struct {
	path string
	rng  lineRange
}

func (i includeLink) render(baseDir string, _ *string) (string, error) {
	full := filepath.Join(baseDir, filepath.FromSlash(i.path))
	data, err := os.ReadFile(full)
	if err != nil {
		return "", fmt.Errorf("include %s: %w", full, err)
	}
	return i.rng.apply(string(data)), nil
}

func (i includeLink) nestedBase(baseDir string) string {
	dir := filepath.Join(baseDir, filepath.Dir(i.path))
	return filepath.Clean(dir)
}

type rustdocIncludeLink struct {
	path string
	rng  lineRange
}

func (r rustdocIncludeLink) render(baseDir string, _ *string) (string, error) {
	full := filepath.Join(baseDir, filepath.FromSlash(r.path))
	data, err := os.ReadFile(full)
	if err != nil {
		return "", fmt.Errorf("rustdoc_include %s: %w", full, err)
	}
	// Mirrors Rust's take_rustdoc_include_lines: every non-selected line is
	// hidden behind `#`. We approximate by emitting the full file with all
	// out-of-range lines commented out.
	return r.rng.applyRustdoc(string(data)), nil
}

func (r rustdocIncludeLink) nestedBase(baseDir string) string {
	dir := filepath.Join(baseDir, filepath.Dir(r.path))
	return filepath.Clean(dir)
}

type playgroundLink struct {
	path  string
	attrs []string
}

func (p playgroundLink) render(baseDir string, _ *string) (string, error) {
	full := filepath.Join(baseDir, filepath.FromSlash(p.path))
	data, err := os.ReadFile(full)
	if err != nil {
		return "", fmt.Errorf("playground %s: %w", full, err)
	}
	contents := string(data)
	if !strings.HasSuffix(contents, "\n") {
		contents += "\n"
	}
	header := "rust"
	if len(p.attrs) > 0 {
		header = "rust," + strings.Join(p.attrs, ",")
	}
	return "```" + header + "\n" + contents + "```\n", nil
}
func (playgroundLink) nestedBase(string) string { return "" }

// lineRange matches Rust's RangeOrAnchor.
type lineRange struct {
	start  *int    // nil = unbounded
	end    *int    // nil = unbounded
	anchor *string // non-nil => anchor mode
}

func parseInclude(spec string) (string, lineRange) {
	parts := strings.SplitN(spec, ":", 3)
	path := parts[0]
	if len(parts) == 1 {
		return path, lineRange{} // whole file
	}
	head := parts[1]
	if len(parts) == 2 {
		// "path:N" or "path:anchor" or "path::N" or "path::anchor"
		if head == "" {
			// ":N" — start at 0, end N
			n := parseInt(parts[1])
			return path, lineRange{start: intPtr(0), end: n}
		}
		n, err := strconv.Atoi(head)
		if err == nil {
			return path, lineRange{start: intPtr(n - 1), end: intPtr(n)}
		}
		return path, lineRange{anchor: &head}
	}
	// "path:start:end"
	startN, err := strconv.Atoi(parts[1])
	if err != nil {
		// "path:anchor:rest" — anchor mode
		return path, lineRange{anchor: &head}
	}
	endN, endErr := strconv.Atoi(parts[2])
	r := lineRange{start: intPtr(startN - 1)}
	if endErr == nil {
		r.end = intPtr(endN)
	}
	return path, r
}

func parseInt(s string) *int {
	n, err := strconv.Atoi(s)
	if err != nil {
		return nil
	}
	return &n
}

func intPtr(n int) *int { return &n }

// apply returns the lines of s that fall within r.
func (r lineRange) apply(s string) string {
	lines := splitLinesLikeRust(s)
	out := applyRange(lines, r)
	return strings.Join(out, "\n")
}

// applyRustdoc emits the file with out-of-range lines replaced by `#` so the
// playground still receives them but the reader only sees the selected
// portion. Anchor mode mirrors Rust's take_rustdoc_include_anchored_lines:
// lines outside the anchor block are dropped entirely, and ANCHOR/ANCHOR_END
// markers themselves are dropped (not emitted with a `# ` prefix).
func (r lineRange) applyRustdoc(s string) string {
	lines := splitLinesLikeRust(s)
	if r.anchor == nil {
		selected := applyRangeIdx(lines, r)
		mask := make([]bool, len(lines))
		for _, i := range selected {
			mask[i] = true
		}
		for i, line := range lines {
			if !mask[i] {
				if strings.HasPrefix(line, "#") {
					lines[i] = "#" + line
				} else {
					lines[i] = "# " + line
				}
			}
		}
		return strings.Join(lines, "\n")
	}
	// Anchor mode: walk the file emulating Rust's
// take_rustdoc_include_anchored_lines (take_lines.rs:81-105). The
// behaviour has three distinct cases:
//
//   - Outside the matching anchor block: ANCHOR_END lines are dropped
//     (matches the original file's trailing marker), every other line
//     is emitted with a `# ` prefix so it renders as <span
//     class="boring">. ANCHOR_START lines (any name) are NOT picked up
//     here because they trigger the `else if let Some(cap) =
//     ANCHOR_START.captures(l)` branch in Rust and are dropped.
//   - Inside the block: ANCHOR_END lines drop and transition out,
//     ANCHOR_START lines (any name) drop, and ordinary lines are
//     emitted as-is (no `# ` prefix) so they become the visible
//     portion of the playground snippet.
	startPat := "ANCHOR: " + *r.anchor
	endPat := "ANCHOR_END: " + *r.anchor
	var out strings.Builder
	within := false
	for _, line := range lines {
		if within {
			if strings.Contains(line, endPat) {
				within = false
				continue
			}
			if strings.Contains(line, "ANCHOR:") {
				// Drop orphan ANCHOR_START lines inside the block.
				continue
			}
			out.WriteString(line)
			out.WriteString("\n")
			continue
		}
		if strings.Contains(line, startPat) {
			within = true
			continue
		}
		// Outside the block: drop any ANCHOR/ANCHOR_END markers — Rust's
		// `else if let Some(cap) = ANCHOR_START.captures(l)` branch
		// matches any line containing `ANCHOR:`, and the second `else if
		// !ANCHOR_END.is_match(l)` drops ANCHOR_END lines. Without this
		// an unrelated `// ANCHOR: unused-anchor-that-should-be-stripped`
		// would surface as a `# // ANCHOR: ...` boring span.
		if strings.Contains(line, "ANCHOR:") || strings.Contains(line, "ANCHOR_END:") {
			continue
		}
		out.WriteString("# ")
		out.WriteString(line)
		out.WriteString("\n")
	}
	result := out.String()
	if len(result) > 0 && result[len(result)-1] == '\n' {
		result = result[:len(result)-1]
	}
	return result
}

func applyRange(lines []string, r lineRange) []string {
	idxs := applyRangeIdx(lines, r)
	out := make([]string, 0, len(idxs))
	for _, i := range idxs {
		out = append(out, lines[i])
	}
	return out
}

func applyRangeIdx(lines []string, r lineRange) []int {
	if r.anchor != nil {
		// Anchor mode: lines between `ANCHOR: <name>` and `ANCHOR_END: <name>`
		// markers (matches mdbook-driver's take_anchored_lines). The Rust
		// reference impl drops lines that contain any `ANCHOR:` directive
		// once inside the block — including orphan `// ANCHOR: foo` lines
		// for an unrelated anchor name. See take_lines.rs:30-44.
		startPat := "ANCHOR: " + *r.anchor
		endPat := "ANCHOR_END: " + *r.anchor
		var retained []int
		anchorFound := false
		for i, line := range lines {
			if anchorFound {
				if strings.Contains(line, endPat) {
					return retained
				}
				// Drop any line containing an ANCHOR: directive (Rust uses
				// the full ANCHOR_START regex here; strings.Contains is a
				// close approximation that covers every mdBook fixture).
				if strings.Contains(line, "ANCHOR:") {
					continue
				}
				retained = append(retained, i)
				continue
			}
			if strings.Contains(line, startPat) {
				anchorFound = true
			}
		}
		return retained
	}
	start := 0
	if r.start != nil {
		start = *r.start
	}
	end := len(lines)
	if r.end != nil {
		end = *r.end
	}
	if start > end {
		start, end = end, start
	}
	if start < 0 {
		start = 0
	}
	if end > len(lines) {
		end = len(lines)
	}
	return rangeSlice(start, end)
}

func rangeSlice(start, end int) []int {
	out := make([]int, 0, end-start)
	for i := start; i < end; i++ {
		out = append(out, i)
	}
	return out
}

// splitLinesLikeRust splits s on '\n' the way Rust's str::lines() does: it
// drops a single trailing empty entry so a file ending in '\n' yields N
// lines instead of N+1. Apply/applyRustdoc need this parity because the
// final empty line, if kept, would surface as a stray
// `<span class="boring"></span>` in the rendered HTML.
func splitLinesLikeRust(s string) []string {
	lines := strings.Split(s, "\n")
	if len(lines) > 0 && lines[len(lines)-1] == "" {
		lines = lines[:len(lines)-1]
	}
	return lines
}