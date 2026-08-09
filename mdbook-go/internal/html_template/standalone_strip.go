package html_template

import (
	"regexp"
	"strings"
)

// standaloneActionRegex matches a Go-template action that is the only non-blank
// content on its line. The pattern tolerates leading and trailing whitespace on
// the line, but the line must not contain any other text outside the action.
//
// We use this to emulate Handlebars' standalone-tag whitespace-stripping rule,
// which the old hbs engine applied (it strips both the leading and trailing
// line whenever a block action like {{#if}} … {{/if}} sits on its own line).
// html/template does not strip anything, so a preprocessor is required to keep
// the rendered output readable.
//
// The full set of Go-template actions we strip: {{if}}, {{else}}, {{end}},
// {{range}}, {{template}}, {{with}}, and their {{- … -}} whitespace-trimming
// variants. {{- …}} and {{… -}} are passed through as-is — they do not affect
// the surrounding line.
var standaloneActionRegex = regexp.MustCompile(`^[ \t]*\{\{[ \t]*\^-?[ \t]*(if|else|end|range|template|with)[^}]*\}\}[ \t]*$`)

// stripStandaloneLines returns src with every line that contains only a
// single block action removed. Blank lines that survive untouched. The function
// is intentionally minimal: it only triggers on the narrow regex above and
// leaves all other content unchanged.
//
// This is a heuristic — Handlebars' standalone rules are subtly different
// (partials preserve call-site indentation, etc.). For the production .html
// files we author, the simpler rule is sufficient.
func stripStandaloneLines(src string) string {
	// strings.Split yields a trailing "" element for a source that ends in
	// "\n"; writing it out would append a spurious blank line, so drop it and
	// let the loop below re-add the one final newline.
	parts := strings.Split(src, "\n")
	if len(parts) > 0 && parts[len(parts)-1] == "" {
		parts = parts[:len(parts)-1]
	}
	var b strings.Builder
	b.Grow(len(src))
	for _, line := range parts {
		if standaloneActionRegex.MatchString(line) {
			continue
		}
		b.WriteString(line)
		b.WriteByte('\n')
	}
	out := b.String()
	// Strip the trailing newline we always add, to keep src ending like the
	// input did.
	if !strings.HasSuffix(src, "\n") && strings.HasSuffix(out, "\n") {
		out = strings.TrimSuffix(out, "\n")
	}
	return out
}
