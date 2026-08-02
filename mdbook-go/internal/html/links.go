package html

import (
	"regexp"
	"strings"
)

// schemeLink matches an absolute URL such as `https:` or `mailto:`.
var schemeLink = regexp.MustCompile(`^[a-z][a-z0-9+.-]*:`)

// FixLink rewrites a Markdown link so it points at the rendered page. Fragment
// links and absolute URLs are returned untouched; `.md` becomes `.html` and any
// anchor is preserved. Ported from fix_link in
// crates/mdbook-html/src/html/tree.rs.
//
// The Rust version applies the unanchored regex `(?P<link>.*)\.md(?P<anchor>#.*)?`
// with a greedy prefix, so it latches onto the LAST `.md` in the string and
// discards whatever follows the match. `a.md.bak` therefore becomes `a.html`.
// That quirk is reproduced here on purpose.
func FixLink(link string) string {
	if strings.HasPrefix(link, "#") {
		return link
	}
	if schemeLink.MatchString(link) {
		return link
	}
	i := strings.LastIndex(link, ".md")
	if i < 0 {
		return link
	}
	anchor := ""
	if rest := link[i+len(".md"):]; strings.HasPrefix(rest, "#") {
		anchor = rest
	}
	return link[:i] + ".html" + anchor
}
