package html

import (
	"regexp"
	"strings"

	"github.com/yuin/goldmark/ast"
)

// Admonition icons come from GitHub's octicons (MIT), copied verbatim from
// crates/mdbook-html/src/html/admonitions.rs so the markup matches byte for
// byte.
const (
	iconNote      = `<path d="M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8Zm8-6.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM6.5 7.75A.75.75 0 0 1 7.25 7h1a.75.75 0 0 1 .75.75v2.75h.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5h.25v-2h-.25a.75.75 0 0 1-.75-.75ZM8 6a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"></path>`
	iconTip       = `<path d="M8 1.5c-2.363 0-4 1.69-4 3.75 0 .984.424 1.625.984 2.304l.214.253c.223.264.47.556.673.848.284.411.537.896.621 1.49a.75.75 0 0 1-1.484.211c-.04-.282-.163-.547-.37-.847a8.456 8.456 0 0 0-.542-.68c-.084-.1-.173-.205-.268-.32C3.201 7.75 2.5 6.766 2.5 5.25 2.5 2.31 4.863 0 8 0s5.5 2.31 5.5 5.25c0 1.516-.701 2.5-1.328 3.259-.095.115-.184.22-.268.319-.207.245-.383.453-.541.681-.208.3-.33.565-.37.847a.751.751 0 0 1-1.485-.212c.084-.593.337-1.078.621-1.489.203-.292.45-.584.673-.848.075-.088.147-.173.213-.253.561-.679.985-1.32.985-2.304 0-2.06-1.637-3.75-4-3.75ZM5.75 12h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1 0-1.5ZM6 15.25a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 0 1.5h-2.5a.75.75 0 0 1-.75-.75Z"></path>`
	iconImportant = `<path d="M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v9.5A1.75 1.75 0 0 1 14.25 13H8.06l-2.573 2.573A1.458 1.458 0 0 1 3 14.543V13H1.75A1.75 1.75 0 0 1 0 11.25Zm1.75-.25a.25.25 0 0 0-.25.25v9.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h6.5a.25.25 0 0 0 .25-.25v-9.5a.25.25 0 0 0-.25-.25Zm7 2.25v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 9a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path>`
	iconWarning   = `<path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"></path>`
	iconCaution   = `<path d="M4.47.22A.749.749 0 0 1 5 0h6c.199 0 .389.079.53.22l4.25 4.25c.141.14.22.331.22.53v6a.749.749 0 0 1-.22.53l-4.25 4.25A.749.749 0 0 1 11 16H5a.749.749 0 0 1-.53-.22L.22 11.53A.749.749 0 0 1 0 11V5c0-.199.079-.389.22-.53Zm.84 1.28L1.5 5.31v5.38l3.81 3.81h5.38l3.81-3.81V5.31L10.69 1.5ZM8 4a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 8 4Zm0 8a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"></path>`
)

// admonition describes one recognised `> [!KIND]` block-quote variant.
type admonition struct {
	class string
	icon  string
	title string
}

// selectTag maps a marker to its class, icon and title, mirroring select_tag in
// crates/mdbook-html/src/html/admonitions.rs.
func selectTag(marker string) (admonition, bool) {
	switch marker {
	case "NOTE":
		return admonition{"note", iconNote, "Note"}, true
	case "TIP":
		return admonition{"tip", iconTip, "Tip"}, true
	case "IMPORTANT":
		return admonition{"important", iconImportant, "Important"}, true
	case "WARNING":
		return admonition{"warning", iconWarning, "Warning"}, true
	case "CAUTION":
		return admonition{"caution", iconCaution, "Caution"}, true
	}
	return admonition{}, false
}

// blockquote emits a plain <blockquote>, or the admonition variant when the
// quote opens with a `[!KIND]` marker on its own line.
func (b *builder) blockquote(n *ast.Blockquote) error {
	kind, marker := admonition{}, false
	if b.opts.Admonitions {
		kind, marker = b.admonitionKind(n)
	}
	if !marker {
		return b.wrap("blockquote", n, nil)
	}

	b.push(NewElement("blockquote").withClass("blockquote-tag blockquote-tag-" + kind.class))
	title := b.push(NewElement("p").withClass("blockquote-tag-title"))
	svg := NewElement("svg")
	svg.El.SetAttr("viewbox", "0 0 16 16")
	svg.El.SetAttr("width", "18")
	svg.El.SetAttr("height", "18")
	svg.Append(NewRawData(kind.icon))
	title.Append(svg)
	title.Append(NewText(kind.title))
	b.pop()

	if err := b.walk(n); err != nil {
		return err
	}
	b.pop()
	return nil
}

// admonitionMarker matches a `[!KIND]` line that opens an admonition.
var admonitionMarker = regexp.MustCompile(`^\[!([A-Za-z]+)\][ \t]*$`)

// admonitionKind inspects the first line of a block quote for a `[!KIND]`
// marker. When one is found the inline nodes covering that line are removed
// from the AST so the marker is not rendered as text.
func (b *builder) admonitionKind(n *ast.Blockquote) (admonition, bool) {
	para, _ := n.FirstChild().(*ast.Paragraph)
	if para == nil || para.Lines().Len() == 0 {
		return admonition{}, false
	}
	firstLine := para.Lines().At(0)
	raw := strings.TrimRight(string(firstLine.Value(b.source)), "\r\n")
	m := admonitionMarker.FindStringSubmatch(raw)
	if m == nil {
		return admonition{}, false
	}
	kind, ok := selectTag(m[1])
	if !ok {
		return admonition{}, false
	}

	// goldmark splits `[!NOTE]` into several text nodes because it first tries
	// to read it as a link. Drop every inline node that falls inside the marker
	// line, up to and including the one carrying its line break.
	for child := para.FirstChild(); child != nil; {
		next := child.NextSibling()
		text, isText := child.(*ast.Text)
		if !isText || text.Segment.Stop > firstLine.Stop {
			break
		}
		para.RemoveChild(para, child)
		if text.SoftLineBreak() || text.HardLineBreak() {
			break
		}
		child = next
	}
	if para.FirstChild() == nil {
		n.RemoveChild(n, para)
	}
	return kind, true
}
