// helper.go holds the HTML escaping and identifier helpers, direct ports of
// crates/mdbook-core/src/utils/html.rs and crates/mdbook-html/src/utils.rs so
// the two implementations stay byte-compatible. (Moved here from internal/utils
// on 2026-08-09.)
package html

import (
	"fmt"
	"strings"
	"unicode"
)

// EscapeHTML escapes `<`, `>` and `&`, matching escape_html in
// crates/mdbook-core/src/utils/html.rs. Quotes are deliberately left alone.
func EscapeHTML(text string) string {
	if !strings.ContainsAny(text, "<>&") {
		return text
	}
	var b strings.Builder
	b.Grow(len(text) + 8)
	for i := 0; i < len(text); i++ {
		switch text[i] {
		case '<':
			b.WriteString("&lt;")
		case '>':
			b.WriteString("&gt;")
		case '&':
			b.WriteString("&amp;")
		default:
			b.WriteByte(text[i])
		}
	}
	return b.String()
}

// EscapeHTMLAttribute escapes `<`, `>`, `'`, `"`, `\` and `&`, matching
// escape_html_attribute in crates/mdbook-core/src/utils/html.rs.
func EscapeHTMLAttribute(text string) string {
	if !strings.ContainsAny(text, "<>'\"\\&") {
		return text
	}
	var b strings.Builder
	b.Grow(len(text) + 16)
	for i := 0; i < len(text); i++ {
		switch text[i] {
		case '<':
			b.WriteString("&lt;")
		case '>':
			b.WriteString("&gt;")
		case '\'':
			b.WriteString("&#39;")
		case '"':
			b.WriteString("&quot;")
		case '\\':
			b.WriteString("&#92;")
		case '&':
			b.WriteString("&amp;")
		default:
			b.WriteByte(text[i])
		}
	}
	return b.String()
}

// IDFromContent slugifies heading text the same way id_from_content does in
// crates/mdbook-html/src/utils.rs: trim, lowercase, keep alphanumerics plus
// `_` and `-`, turn whitespace into `-`, drop everything else.
func IDFromContent(content string) string {
	var b strings.Builder
	b.Grow(len(content))
	for _, ch := range strings.ToLower(strings.TrimSpace(content)) {
		switch {
		case unicode.IsLetter(ch) || unicode.IsDigit(ch) || ch == '_' || ch == '-':
			b.WriteRune(ch)
		case unicode.IsSpace(ch):
			b.WriteRune('-')
		}
	}
	return b.String()
}

// IDSet tracks the identifiers handed out so far so duplicates can be
// disambiguated. The zero value is not usable; call NewIDSet.
type IDSet struct {
	used map[string]struct{}
}

// NewIDSet returns an empty identifier set.
func NewIDSet() *IDSet {
	return &IDSet{used: make(map[string]struct{})}
}

// Unique returns id if it has not been handed out yet, otherwise it appends
// `-1`, `-2`, ... until an unused candidate is found. This mirrors unique_id in
// crates/mdbook-html/src/utils.rs.
func (s *IDSet) Unique(id string) string {
	if _, ok := s.used[id]; !ok {
		s.used[id] = struct{}{}
		return id
	}
	for counter := 1; ; counter++ {
		candidate := fmt.Sprintf("%s-%d", id, counter)
		if _, ok := s.used[candidate]; !ok {
			s.used[candidate] = struct{}{}
			return candidate
		}
	}
}
