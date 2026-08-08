package html

import (
	"regexp"
	"strings"
)

// boringLines matches an optionally indented `#`-prefixed Rust line. Ported
// from BORING_LINES_REGEX in crates/mdbook-html/src/html/hide_lines.rs.
var boringLines = regexp.MustCompile(`^(\s*)#(.?)(.*)$`)

// splitLines splits on '\n' the way Rust's str::lines does: the trailing
// newline does not produce an extra empty element, and a '\r' before the
// newline is dropped.
func splitLines(text string) []string {
	if text == "" {
		return nil
	}
	text = strings.TrimSuffix(text, "\n")
	lines := strings.Split(text, "\n")
	for i, l := range lines {
		lines[i] = strings.TrimSuffix(l, "\r")
	}
	return lines
}

// hideLinesRust wraps `# `-prefixed Rust lines in `<span class="boring">`,
// stripping the marker. `##` escapes to a literal `#`.
func hideLinesRust(text string) []*Node {
	var out []*Node
	lines := splitLines(text)
	for i, line := range lines {
		newline := "\n"
		if i == len(lines)-1 {
			newline = ""
		}
		if caps := boringLines.FindStringSubmatch(line); caps != nil {
			if caps[2] == "#" {
				out = append(out, NewText(caps[1]+caps[2]+caps[3]+newline))
				continue
			}
			if caps[2] == "" || caps[2] == " " {
				span := NewElement("span")
				span.El.SetAttr("class", "boring")
				span.Append(NewText(caps[1] + caps[3] + newline))
				out = append(out, span)
				continue
			}
		}
		out = append(out, NewText(line+newline))
	}
	return out
}

// hideLinesWithPrefix wraps lines whose first non-space run starts with prefix.
// Unlike the Rust variant every emitted line keeps its trailing newline, which
// matches hide_lines_with_prefix.
func hideLinesWithPrefix(content, prefix string) []*Node {
	var out []*Node
	for _, line := range splitLines(content) {
		if strings.HasPrefix(strings.TrimLeft(line, " \t\n\v\f\r"), prefix) {
			pos := strings.Index(line, prefix)
			ws, rest := line[:pos], line[pos+len(prefix):]
			span := NewElement("span")
			span.El.SetAttr("class", "boring")
			span.Append(NewText(ws + rest + "\n"))
			out = append(out, span)
			continue
		}
		out = append(out, NewText(line+"\n"))
	}
	return out
}

// partitionRustSource and rustHeader were removed along with the Rust
// playground feature: their only consumer was wrapRustMain, which fed Rust
// snippets into the playground iframe.
