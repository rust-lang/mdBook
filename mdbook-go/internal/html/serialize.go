package html

import (
	"strings"

)

// Serialize writes the tree as HTML. It is a port of
// crates/mdbook-html/src/html/serialize.rs, including its pretty-printing rule:
// a newline is emitted before the start tag of a block-level element (unless
// the output is empty or already ends in a newline) and after its end tag.
func Serialize(root *Node) string {
	var b strings.Builder
	SerializeInto(root, &b)
	return b.String()
}

// SerializeInto appends the tree to an existing buffer. The print page relies
// on this: because the pretty printer looks at what has already been written,
// serializing chapter after chapter into one buffer is what puts the page-break
// separators on their own lines.
func SerializeInto(root *Node, b *strings.Builder) {
	serializeNode(root, b)
}

func serializeNode(n *Node, b *strings.Builder) {
	switch n.Kind {
	case KindFragment:
	case KindElement:
		serializeStart(n.El, b)
	case KindText:
		b.WriteString(EscapeHTML(n.Text))
	case KindComment:
		b.WriteString("<!--")
		b.WriteString(n.Text)
		b.WriteString("-->")
	case KindRawData:
		b.WriteString(n.Text)
	}
	for _, c := range n.Children {
		serializeNode(c, b)
	}
	if n.Kind == KindElement {
		serializeEnd(n.El, b)
	}
}

// wantsPrettyHTMLNewline reports whether the element should be surrounded by
// newlines for readability.
func wantsPrettyHTMLNewline(name string) bool {
	switch name {
	case "blockquote", "dd", "div", "dl", "dt",
		"h1", "h2", "h3", "h4", "h5", "h6",
		"hr", "li", "ol", "p", "pre", "table", "tbody", "thead", "tr", "ul":
		return true
	}
	return false
}

func serializeStart(el *Element, b *strings.Builder) {
	if wantsPrettyHTMLNewline(el.Name) {
		s := b.String()
		if s != "" && !strings.HasSuffix(s, "\n") {
			b.WriteByte('\n')
		}
	}
	b.WriteByte('<')
	b.WriteString(el.Name)
	for _, attr := range el.Attrs {
		b.WriteByte(' ')
		switch attr.NS {
		case NSXML:
			b.WriteString("xml:")
		case NSXMLNS:
			if el.Name != "xmlns" {
				b.WriteString("xmlns:")
			}
		case NSXLink:
			b.WriteString("xlink:")
		}
		b.WriteString(attr.Name)
		b.WriteString("=\"")
		b.WriteString(EscapeHTMLAttribute(attr.Value))
		b.WriteByte('"')
	}
	if el.SelfClosing {
		b.WriteString(" /")
	}
	b.WriteByte('>')
}

func serializeEnd(el *Element, b *strings.Builder) {
	if el.SelfClosing || isVoidElement(el.Name) {
		return
	}
	b.WriteString("</")
	b.WriteString(el.Name)
	b.WriteByte('>')
	if wantsPrettyHTMLNewline(el.Name) {
		b.WriteByte('\n')
	}
}
