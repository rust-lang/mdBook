package html

import (
	"strings"
)

// addHeaderLinks gives every heading (and definition term) a unique id and
// wraps its contents in `<a class="header" href="#id">`. Elements that came
// from literal HTML in the source are left alone. Ported from add_header_links
// in crates/mdbook-html/src/html/tree.rs.
func (b *builder) addHeaderLinks() {
	ids := NewIDSet()
	headings := b.root.Elements(func(name string) bool {
		switch name {
		case "h1", "h2", "h3", "h4", "h5", "h6", "dt":
			return true
		}
		return false
	})
	for _, heading := range headings {
		if heading.El.WasRaw {
			continue
		}
		href := ""
		if id, ok := heading.El.Attr("id"); ok {
			href = "#" + id
		} else {
			id := ids.Unique(IDFromContent(heading.TextContent()))
			heading.El.SetAttr("id", id)
			href = "#" + id
		}
		anchor := NewElement("a")
		anchor.El.SetAttr("class", "header")
		anchor.El.SetAttr("href", href)
		anchor.Children = heading.Children
		for _, c := range anchor.Children {
			c.Parent = anchor
		}
		heading.Children = nil
		heading.Append(anchor)
	}
}

// updateCodeBlocks applies the configurable `hidelines=<prefix>` hidden-line
// markup to every code block. The rust-specific `hide lines` pass (and the
// playground treatment before it) was removed with the other Rust leftovers.
// Ported from update_code_blocks in crates/mdbook-html/src/html/tree.rs
// (minus the playground and rust branches).
func (b *builder) updateCodeBlocks() {
	for _, code := range b.root.Elements(func(name string) bool { return name == "code" }) {
		class, _ := code.El.Attr("class")
		classes := strings.Fields(class)

		hidelinesPrefix, hasPrefix := "", false
		var language string
		for _, c := range classes {
			if strings.HasPrefix(c, "language-") {
				language = strings.TrimPrefix(c, "language-")
			}
			if rest, ok := strings.CutPrefix(c, "hidelines="); ok {
				hidelinesPrefix, hasPrefix = rest, true
			}
		}
		if !hasPrefix {
			if p, ok := b.opts.HideLines[language]; ok {
				hidelinesPrefix, hasPrefix = p, true
			}
		}

		if hasPrefix {
			setCodeChildren(code, hideLinesWithPrefix(codeText(code), hidelinesPrefix))
		}
	}
}

// codeText concatenates the text children of a <code> element.
func codeText(code *Node) string {
	var b strings.Builder
	for _, c := range code.Children {
		if c.Kind == KindText {
			b.WriteString(c.Text)
		}
	}
	return b.String()
}

func setCodeText(code *Node, text string) {
	code.Children = nil
	code.Append(NewText(text))
}

func setCodeChildren(code *Node, children []*Node) {
	code.Children = nil
	for _, c := range children {
		code.Append(c)
	}
}

// replaceChild swaps old for replacement in n's child list.
func (n *Node) replaceChild(old, replacement *Node) {
	for i, c := range n.Children {
		if c == old {
			replacement.Parent = n
			n.Children[i] = replacement
			return
		}
	}
}
