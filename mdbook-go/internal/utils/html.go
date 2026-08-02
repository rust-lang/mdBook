// Package utils holds the small helpers shared by the loader, the markdown
// pipeline and the HTML renderer. Every function here is a direct port of the
// matching Rust helper so the two implementations stay byte-compatible:
//
//	EscapeHTML / EscapeHTMLAttribute -> crates/mdbook-core/src/utils/html.rs
//	PathToRoot                       -> crates/mdbook-core/src/utils/fs.rs
//	IDFromContent / UniqueID         -> crates/mdbook-html/src/utils.rs
package utils

import "strings"

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
