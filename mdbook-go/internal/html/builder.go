package html

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"

	"github.com/yuin/goldmark"
	"github.com/yuin/goldmark/ast"
	"github.com/yuin/goldmark/extension"
	extast "github.com/yuin/goldmark/extension/ast"
	"github.com/yuin/goldmark/parser"
	"github.com/yuin/goldmark/text"
)

// Options controls how a chapter's Markdown is turned into HTML. It is the Go
// equivalent of HtmlRenderOptions in crates/mdbook-html/src/html/mod.rs.
type Options struct {
	// Path is the chapter's source path, used only in diagnostics.
	Path string

	SmartPunctuation bool
	DefinitionLists  bool
	Admonitions      bool
	MathJax          bool

	// HideLines maps a language to the prefix that marks a hidden line.
	HideLines map[string]string
}

// builder turns a goldmark AST into the mdBook node tree.
type builder struct {
	opts   Options
	source []byte
	root   *Node
	cur    *Node

	// footnoteNumbers assigns each footnote its display number and counts how
	// often it was referenced. goldmark identifies references by index only, so
	// footnoteIndexNames maps those back to the labels used in the markup.
	footnoteOrder      []string
	footnoteNumbers    map[string]*footnoteInfo
	footnoteDefs       map[string]*Node
	footnoteIndexNames map[int]string

	// rawStack tracks elements opened by literal HTML in the source so that a
	// later closing tag pops the right node.
	rawStack []*Node
}

type footnoteInfo struct {
	number int
	uses   int
}

// BuildTree parses Markdown and returns the mdBook node tree for it.
func BuildTree(source string, opts Options) (*Node, error) {
	exts := []goldmark.Extender{
		extension.Table,
		extension.Strikethrough,
		extension.TaskList,
		extension.Footnote,
	}
	if opts.DefinitionLists {
		exts = append(exts, extension.DefinitionList)
	}
	if opts.SmartPunctuation {
		exts = append(exts, smartPunctuation())
	}
	md := goldmark.New(
		goldmark.WithExtensions(exts...),
		goldmark.WithParserOptions(parser.WithAttribute()),
	)

	src := []byte(source)
	reader := text.NewReader(src)
	doc := md.Parser().Parse(reader)

	b := &builder{
		opts:               opts,
		source:             src,
		root:               NewFragment(),
		footnoteNumbers:    map[string]*footnoteInfo{},
		footnoteDefs:       map[string]*Node{},
		footnoteIndexNames: footnoteLabels(doc),
	}
	b.cur = b.root
	if err := b.walk(doc); err != nil {
		return nil, err
	}
	b.collectFootnoteDefs()
	b.addHeaderLinks()
	b.updateCodeBlocks()
	return b.root, nil
}

// Render is the convenience wrapper used by the renderer: parse, transform and
// serialize in one step.
func Render(source string, opts Options) (string, error) {
	tree, err := BuildTree(source, opts)
	if err != nil {
		return "", err
	}
	return Serialize(tree), nil
}

// smartPunctuation configures goldmark's typographer to emit the literal
// characters rather than HTML entities. pulldown-cmark's ENABLE_SMART_PUNCTUATION
// produces real code points, and the rest of the pipeline (heading ids, the
// search index) needs to see them as text.
func smartPunctuation() goldmark.Extender {
	return extension.NewTypographer(
		extension.WithTypographicSubstitutions(map[extension.TypographicPunctuation]string{
			extension.LeftSingleQuote:  "‘",
			extension.RightSingleQuote: "’",
			extension.LeftDoubleQuote:  "“",
			extension.RightDoubleQuote: "”",
			extension.EnDash:           "–",
			extension.EmDash:           "—",
			extension.Ellipsis:         "…",
			extension.LeftAngleQuote:   "«",
			extension.RightAngleQuote:  "»",
			extension.Apostrophe:       "’",
		}),
	)
}

// footnoteLabels maps goldmark's numeric footnote indices back to the labels
// written in the source. References are visited before the definition list, so
// the mapping has to be built up front.
func footnoteLabels(doc ast.Node) map[int]string {
	labels := map[int]string{}
	_ = ast.Walk(doc, func(n ast.Node, entering bool) (ast.WalkStatus, error) {
		if !entering {
			return ast.WalkContinue, nil
		}
		if fn, ok := n.(*extast.Footnote); ok {
			labels[fn.Index] = string(fn.Ref)
		}
		return ast.WalkContinue, nil
	})
	return labels
}

// push appends node to the current parent and descends into it.
func (b *builder) push(node *Node) *Node {
	b.cur.Append(node)
	b.cur = node
	return node
}

// pop returns to the parent node.
func (b *builder) pop() {
	if b.cur.Parent != nil {
		b.cur = b.cur.Parent
	}
}

// append adds a leaf without descending.
func (b *builder) append(node *Node) *Node { return b.cur.Append(node) }

func (b *builder) text(n ast.Node) string { return string(n.Text(b.source)) }

// walk drives the AST traversal. Nodes that produce no element of their own
// (documents, text blocks) simply recurse.
func (b *builder) walk(n ast.Node) error {
	for child := n.FirstChild(); child != nil; child = child.NextSibling() {
		if err := b.node(child); err != nil {
			return err
		}
	}
	return nil
}

func (b *builder) node(n ast.Node) error {
	switch node := n.(type) {
	case *ast.Paragraph:
		return b.wrap("p", n, nil)
	case *ast.TextBlock:
		// A tight list item's contents; emitted without a wrapper.
		return b.walk(n)
	case *ast.Heading:
		return b.heading(node)
	case *ast.ThematicBreak:
		b.append(NewElement("hr"))
		return nil
	case *ast.Blockquote:
		return b.blockquote(node)
	case *ast.List:
		return b.list(node)
	case *ast.ListItem:
		return b.wrap("li", n, nil)
	case *ast.FencedCodeBlock:
		return b.codeBlock(node, string(node.Language(b.source)), b.infoString(node))
	case *ast.CodeBlock:
		return b.codeBlock(node, "", "")
	case *ast.HTMLBlock:
		return b.htmlBlock(node)
	case *ast.RawHTML:
		return b.rawHTML(node)
	case *ast.Text:
		return b.textNode(node)
	case *ast.String:
		b.append(NewText(string(node.Value)))
		return nil
	case *ast.CodeSpan:
		return b.wrap("code", n, nil)
	case *ast.Emphasis:
		name := "em"
		if node.Level == 2 {
			name = "strong"
		}
		return b.wrap(name, n, nil)
	case *ast.Link:
		return b.link(node)
	case *ast.AutoLink:
		return b.autoLink(node)
	case *ast.Image:
		return b.image(node)
	case *extast.Strikethrough:
		return b.wrap("del", n, nil)
	case *extast.TaskCheckBox:
		input := NewElement("input")
		input.El.SetAttr("disabled", "")
		input.El.SetAttr("type", "checkbox")
		if node.IsChecked {
			input.El.SetAttr("checked", "")
		}
		b.append(input)
		// goldmark swallows the space that follows the marker; pulldown-cmark
		// keeps it, and it is visible in the rendered list item.
		b.append(NewText(" "))
		return nil
	case *extast.Table:
		return b.table(node)
	case *extast.TableHeader:
		return b.tableSection("thead", node)
	case *extast.TableRow:
		return b.wrap("tr", n, nil)
	case *extast.TableCell:
		return b.tableCell(node)
	case *extast.FootnoteLink:
		return b.footnoteReference(node)
	case *extast.Footnote:
		return b.footnoteDefinition(node)
	case *extast.FootnoteList:
		// Definitions are collected as they are visited and re-emitted at the
		// end of the document by collectFootnoteDefs.
		return b.walk(n)
	case *extast.DefinitionList:
		return b.wrap("dl", n, nil)
	case *extast.DefinitionTerm:
		return b.wrap("dt", n, nil)
	case *extast.DefinitionDescription:
		return b.wrap("dd", n, nil)
	default:
		// Unknown containers still contribute their children.
		return b.walk(n)
	}
}

// wrap pushes an element, walks the children and pops back out.
func (b *builder) wrap(name string, n ast.Node, attrs func(*Element)) error {
	el := NewElement(name)
	if attrs != nil {
		attrs(el.El)
	}
	b.push(el)
	if err := b.walk(n); err != nil {
		return err
	}
	b.pop()
	return nil
}

func (b *builder) heading(n *ast.Heading) error {
	name := "h" + strconv.Itoa(n.Level)
	return b.wrap(name, n, func(el *Element) {
		// `{#id .class key=value}` attributes, enabled by parser.WithAttribute.
		for _, attr := range n.Attributes() {
			key := string(attr.Name)
			switch v := attr.Value.(type) {
			case []byte:
				el.SetAttr(key, string(v))
			case []any:
				parts := make([]string, 0, len(v))
				for _, item := range v {
					if bs, ok := item.([]byte); ok {
						parts = append(parts, string(bs))
					}
				}
				el.SetAttr(key, strings.Join(parts, " "))
			default:
				el.SetAttr(key, fmt.Sprint(v))
			}
		}
	})
}

func (b *builder) list(n *ast.List) error {
	name := "ul"
	var start int
	if n.IsOrdered() {
		name = "ol"
		start = n.Start
	}
	return b.wrap(name, n, func(el *Element) {
		if name == "ol" && start != 1 {
			el.SetAttr("start", strconv.Itoa(start))
		}
	})
}

func (b *builder) table(n *extast.Table) error {
	// mdBook wraps tables so they can scroll horizontally.
	b.push(NewElement("div").withClass("table-wrapper"))
	b.push(NewElement("table"))

	var body *Node
	for child := n.FirstChild(); child != nil; child = child.NextSibling() {
		if _, isHeader := child.(*extast.TableHeader); isHeader {
			if err := b.node(child); err != nil {
				return err
			}
			continue
		}
		if body == nil {
			body = b.push(NewElement("tbody"))
		}
		if err := b.node(child); err != nil {
			return err
		}
	}
	if body != nil {
		b.pop()
	}
	b.pop()
	b.pop()
	return nil
}

func (b *builder) tableSection(name string, n ast.Node) error {
	b.push(NewElement(name))
	b.push(NewElement("tr"))
	if err := b.walk(n); err != nil {
		return err
	}
	b.pop()
	b.pop()
	return nil
}

func (b *builder) tableCell(n *extast.TableCell) error {
	name := "td"
	if _, ok := n.Parent().(*extast.TableHeader); ok {
		name = "th"
	}
	return b.wrap(name, n, func(el *Element) {
		switch n.Alignment {
		case extast.AlignLeft:
			el.SetAttr("style", "text-align: left")
		case extast.AlignRight:
			el.SetAttr("style", "text-align: right")
		case extast.AlignCenter:
			el.SetAttr("style", "text-align: center")
		}
	})
}

func (b *builder) textNode(n *ast.Text) error {
	b.append(NewText(unescapeText(n.Segment.Value(b.source))))
	switch {
	case n.HardLineBreak():
		b.append(NewElement("br"))
	case n.SoftLineBreak():
		b.append(NewText("\n"))
	}
	return nil
}

func (b *builder) link(n *ast.Link) error {
	return b.wrap("a", n, func(el *Element) {
		el.SetAttr("href", FixLink(string(n.Destination)))
		if len(n.Title) > 0 {
			el.SetAttr("title", string(n.Title))
		}
	})
}

func (b *builder) autoLink(n *ast.AutoLink) error {
	url := string(n.URL(b.source))
	if n.AutoLinkType == ast.AutoLinkEmail && !strings.HasPrefix(url, "mailto:") {
		url = "mailto:" + url
	}
	a := NewElement("a")
	a.El.SetAttr("href", url)
	b.push(a)
	b.append(NewText(string(n.Label(b.source))))
	b.pop()
	return nil
}

// image emits the click-to-zoom wrapper mdBook uses:
// <label class="checkbox-label"><input class="checkbox-img" type="checkbox">
// <img …><span class="img-wrapper"><img …></span></label>
func (b *builder) image(n *ast.Image) error {
	img := func() *Node {
		el := NewElement("img")
		el.El.SetAttr("src", FixLink(string(n.Destination)))
		if len(n.Title) > 0 {
			el.El.SetAttr("title", string(n.Title))
		}
		el.El.SetAttr("alt", b.altText(n))
		return el
	}

	label := b.push(NewElement("label").withClass("checkbox-label"))
	input := NewElement("input")
	input.El.SetAttr("class", "checkbox-img")
	input.El.SetAttr("type", "checkbox")
	label.Append(input)
	label.Append(img())
	wrapper := label.Append(NewElement("span").withClass("img-wrapper"))
	wrapper.Append(img())
	b.pop()
	return nil
}

// altText flattens the image's inline children into alt text. Line breaks and
// rules collapse to a single space, matching text_for_img_alt in
// crates/mdbook-html/src/html/tree.rs.
func (b *builder) altText(n ast.Node) string {
	var sb strings.Builder
	var visit func(ast.Node)
	visit = func(node ast.Node) {
		for c := node.FirstChild(); c != nil; c = c.NextSibling() {
			switch v := c.(type) {
			case *ast.Text:
				sb.WriteString(unescapeText(v.Segment.Value(b.source)))
				if v.HardLineBreak() || v.SoftLineBreak() {
					sb.WriteByte(' ')
				}
			case *ast.String:
				sb.Write(v.Value)
			case *ast.RawHTML:
				for i := 0; i < v.Segments.Len(); i++ {
					seg := v.Segments.At(i)
					sb.Write(seg.Value(b.source))
				}
			case *ast.ThematicBreak:
				sb.WriteByte(' ')
			case *extast.TaskCheckBox, *extast.FootnoteLink:
				// Dropped from alt text.
			default:
				visit(c)
			}
		}
	}
	visit(n)
	return sb.String()
}

// infoString returns everything after the language on a fenced code block.
func (b *builder) infoString(n *ast.FencedCodeBlock) string {
	if n.Info == nil {
		return ""
	}
	return string(n.Info.Segment.Value(b.source))
}

func (b *builder) codeBlock(n ast.Node, language, info string) error {
	var body bytes.Buffer
	lines := n.Lines()
	for i := 0; i < lines.Len(); i++ {
		seg := lines.At(i)
		body.Write(seg.Value(b.source))
	}

	pre := b.push(NewElement("pre"))
	code := NewElement("code")
	if class := codeClass(language, info); class != "" {
		code.El.SetAttr("class", class)
	}
	code.Append(NewText(body.String()))
	pre.Append(code)
	b.pop()
	return nil
}

// codeClass builds the `language-x extra classes` attribute. The info string is
// split on spaces, tabs and commas, empty fragments are dropped, and the first
// fragment becomes the language.
func codeClass(language, info string) string {
	fields := strings.FieldsFunc(info, func(r rune) bool {
		return r == ' ' || r == '\t' || r == ','
	})
	if len(fields) == 0 {
		if language == "" {
			return ""
		}
		return "language-" + language
	}
	parts := []string{"language-" + fields[0]}
	parts = append(parts, fields[1:]...)
	return strings.Join(parts, " ")
}

// withClass is a small helper for building elements inline.
func (n *Node) withClass(class string) *Node {
	n.El.SetAttr("class", class)
	return n
}
