package html

import (
	"strings"

	"github.com/yuin/goldmark/ast"
	xhtml "golang.org/x/net/html"
)

// htmlBlock inserts a block of literal HTML from the Markdown source.
func (b *builder) htmlBlock(n *ast.HTMLBlock) error {
	var sb strings.Builder
	lines := n.Lines()
	for i := 0; i < lines.Len(); i++ {
		seg := lines.At(i)
		sb.Write(seg.Value(b.source))
	}
	if n.HasClosure() {
		sb.Write(n.ClosureLine.Value(b.source))
	}
	return b.rawFragment(sb.String())
}

// rawHTML inserts an inline run of literal HTML from the Markdown source.
func (b *builder) rawHTML(n *ast.RawHTML) error {
	var sb strings.Builder
	for i := 0; i < n.Segments.Len(); i++ {
		seg := n.Segments.At(i)
		sb.Write(seg.Value(b.source))
	}
	return b.rawFragment(sb.String())
}

// rawFragment tokenizes literal HTML and folds it into the node tree. mdBook
// does the same with html5ever so that later passes can rewrite links and so
// that void elements serialize correctly. Because Markdown hands us opening and
// closing tags in separate events, the element stack is kept on the builder
// across calls.
func (b *builder) rawFragment(fragment string) error {
	z := xhtml.NewTokenizer(strings.NewReader(fragment))
	for {
		switch z.Next() {
		case xhtml.ErrorToken:
			return nil

		case xhtml.TextToken:
			text := string(z.Text())
			// <script> and <style> bodies must not be re-escaped.
			if b.cur.Kind == KindElement && (b.cur.El.Name == "script" || b.cur.El.Name == "style") {
				b.append(NewRawData(text))
			} else {
				b.append(NewText(text))
			}

		case xhtml.CommentToken:
			b.append(NewComment(string(z.Text())))

		case xhtml.DoctypeToken:
			b.append(NewRawData("<!DOCTYPE " + string(z.Text()) + ">"))

		case xhtml.StartTagToken:
			node := b.rawElement(z)
			if isVoidElement(node.El.Name) {
				b.append(node)
				continue
			}
			b.push(node)
			b.rawStack = append(b.rawStack, node)

		case xhtml.SelfClosingTagToken:
			node := b.rawElement(z)
			node.El.SelfClosing = !isVoidElement(node.El.Name)
			b.append(node)

		case xhtml.EndTagToken:
			name, _ := z.TagName()
			b.popRaw(string(name))
		}
	}
}

// rawElement builds an element node from the tokenizer's current tag, applying
// the same link rewriting mdBook applies to literal HTML.
func (b *builder) rawElement(z *xhtml.Tokenizer) *Node {
	name, hasAttr := z.TagName()
	node := NewElement(string(name))
	node.El.WasRaw = true
	for hasAttr {
		var key, val []byte
		key, val, hasAttr = z.TagAttr()
		attr := Attr{Name: string(key), Value: string(val)}
		if rest, ok := strings.CutPrefix(attr.Name, "xlink:"); ok {
			attr.NS, attr.Name = NSXLink, rest
		} else if rest, ok := strings.CutPrefix(attr.Name, "xml:"); ok {
			attr.NS, attr.Name = NSXML, rest
		} else if rest, ok := strings.CutPrefix(attr.Name, "xmlns:"); ok {
			attr.NS, attr.Name = NSXMLNS, rest
		}
		if node.El.Name == "a" && attr.Name == "href" {
			attr.Value = FixLink(attr.Value)
		}
		node.El.Attrs = append(node.El.Attrs, attr)
	}
	return node
}

// popRaw closes the most recent matching raw element. Unbalanced closing tags
// are ignored, as they are in the Rust renderer.
func (b *builder) popRaw(name string) {
	for i := len(b.rawStack) - 1; i >= 0; i-- {
		if b.rawStack[i].El.Name != name {
			continue
		}
		b.cur = b.rawStack[i].Parent
		b.rawStack = b.rawStack[:i]
		return
	}
}
