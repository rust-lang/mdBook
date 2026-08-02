package html

import (
	"strings"

	"mdbook-go/internal/fontawesome"
	"mdbook-go/internal/utils"
)

// addHeaderLinks gives every heading (and definition term) a unique id and
// wraps its contents in `<a class="header" href="#id">`. Elements that came
// from literal HTML in the source are left alone. Ported from add_header_links
// in crates/mdbook-html/src/html/tree.rs.
func (b *builder) addHeaderLinks() {
	ids := utils.NewIDSet()
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
			id := ids.Unique(utils.IDFromContent(heading.TextContent()))
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

// updateCodeBlocks applies the Rust playground treatment and then the hidden
// line markup to every code block. Ported from update_code_blocks in
// crates/mdbook-html/src/html/tree.rs.
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

		isEditable := hasClass(classes, "editable")
		isPlayground := hasClass(classes, "language-rust") &&
			((!hasClass(classes, "ignore") &&
				!hasClass(classes, "noplayground") &&
				!hasClass(classes, "noplaypen") &&
				b.opts.Playground.Runnable) ||
				hasClass(classes, "mdbook-runnable"))

		if isPlayground {
			if !hasEditionClass(classes) && b.opts.Edition != "" {
				class = strings.TrimSpace(class + " edition" + b.opts.Edition)
				code.El.SetAttr("class", class)
			}
			if !(b.opts.Playground.Editable && isEditable) {
				if wrapped, ok := wrapRustMain(codeText(code)); ok {
					setCodeText(code, wrapped)
				}
			}
			if code.Parent != nil && code.Parent.Kind == KindElement && code.Parent.El.Name == "pre" {
				code.Parent.El.SetAttr("class", "playground")
			}
		}

		if language == "rust" {
			setCodeChildren(code, hideLinesRust(codeText(code)))
		} else if hasPrefix {
			setCodeChildren(code, hideLinesWithPrefix(codeText(code), hidelinesPrefix))
		}
	}
}

func hasClass(classes []string, want string) bool {
	for _, c := range classes {
		if c == want {
			return true
		}
	}
	return false
}

func hasEditionClass(classes []string) bool {
	for _, c := range classes {
		if strings.HasPrefix(c, "edition") {
			return true
		}
	}
	return false
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

// convertFontAwesome replaces empty `<i class="fa-…">` elements with the inline
// SVG span mdBook emits. Ported from convert_fontawesome in
// crates/mdbook-html/src/html/tree.rs.
func (b *builder) convertFontAwesome() {
	for _, node := range b.root.Elements(func(name string) bool { return name == "i" }) {
		if len(node.Children) != 0 {
			continue
		}
		class, ok := node.El.Attr("class")
		if !ok {
			continue
		}
		iconType, name := fontawesome.Regular, ""
		var extra []string
		for _, c := range strings.Fields(class) {
			switch c {
			case "fa", "fa-regular":
				iconType = fontawesome.Regular
			case "fas", "fa-solid":
				iconType = fontawesome.Solid
			case "fab", "fa-brands":
				iconType = fontawesome.Brands
			default:
				if n, found := strings.CutPrefix(c, "fa-"); found && name == "" {
					name = n
				} else {
					extra = append(extra, c)
				}
			}
		}
		if name == "" {
			continue
		}
		svg, err := fontawesome.SVG(iconType, name)
		if err != nil {
			continue
		}
		span := NewElement("span")
		span.El.SetAttr("class", strings.Join(append([]string{"fa-svg"}, extra...), " "))
		span.Append(NewRawData(svg))
		node.Parent.replaceChild(node, span)
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
