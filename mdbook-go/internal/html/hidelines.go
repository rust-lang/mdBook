package html

import (
	"strings"
)

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
