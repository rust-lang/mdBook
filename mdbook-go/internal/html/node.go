// Package html turns Markdown into the exact HTML tree that mdBook's Rust
// renderer produces. It is a port of crates/mdbook-html/src/html/, which builds
// an intermediate node tree from the Markdown parser instead of emitting HTML
// directly, so that later passes (header anchors, code-block rewriting, link
// fixing, the print page) can operate structurally.
package html

// Kind discriminates the node variants, mirroring the Rust `Node` enum.
type Kind int

// Node kinds.
const (
	// KindFragment is the invisible tree root.
	KindFragment Kind = iota
	// KindElement is an HTML element.
	KindElement
	// KindText is a text node; it is escaped on serialization.
	KindText
	// KindComment is an HTML comment.
	KindComment
	// KindRawData is verbatim output, used for <script>/<style> bodies.
	KindRawData
)

// Namespace identifies the (rare) namespaced attributes mdBook emits.
type Namespace int

// Attribute namespaces.
const (
	NSNone Namespace = iota
	NSXML
	NSXMLNS
	NSXLink
)

// Attr is a single attribute. Attributes keep insertion order, which the
// serializer relies on for byte-compatible output.
type Attr struct {
	NS    Namespace
	Name  string
	Value string
}

// Element is the payload of a KindElement node.
type Element struct {
	Name  string
	Attrs []Attr
	// SelfClosing marks raw HTML written as `<tag />`.
	SelfClosing bool
	// WasRaw marks elements that came from literal HTML in the Markdown
	// source. Such elements are left alone by the header-anchor pass.
	WasRaw bool
}

// Attr returns the value of the named attribute in the default namespace.
func (e *Element) Attr(name string) (string, bool) {
	for _, a := range e.Attrs {
		if a.NS == NSNone && a.Name == name {
			return a.Value, true
		}
	}
	return "", false
}

// SetAttr inserts or replaces an attribute in the default namespace.
func (e *Element) SetAttr(name, value string) {
	for i := range e.Attrs {
		if e.Attrs[i].NS == NSNone && e.Attrs[i].Name == name {
			e.Attrs[i].Value = value
			return
		}
	}
	e.Attrs = append(e.Attrs, Attr{Name: name, Value: value})
}

// Node is one node of the document tree.
type Node struct {
	Kind     Kind
	El       *Element
	Text     string
	Parent   *Node
	Children []*Node
}

// NewFragment returns an empty tree root.
func NewFragment() *Node { return &Node{Kind: KindFragment} }

// NewElement returns a detached element node.
func NewElement(name string) *Node {
	return &Node{Kind: KindElement, El: &Element{Name: name}}
}

// NewText returns a detached text node.
func NewText(text string) *Node { return &Node{Kind: KindText, Text: text} }

// NewRawData returns a detached verbatim node.
func NewRawData(data string) *Node { return &Node{Kind: KindRawData, Text: data} }

// NewComment returns a detached comment node.
func NewComment(text string) *Node { return &Node{Kind: KindComment, Text: text} }

// Append adds child as the last child of n and returns child.
func (n *Node) Append(child *Node) *Node {
	child.Parent = n
	n.Children = append(n.Children, child)
	return child
}

// Prepend adds child as the first child of n and returns child.
func (n *Node) Prepend(child *Node) *Node {
	child.Parent = n
	n.Children = append([]*Node{child}, n.Children...)
	return child
}

// Detach removes n from its parent.
func (n *Node) Detach() {
	if n.Parent == nil {
		return
	}
	siblings := n.Parent.Children
	for i, c := range siblings {
		if c == n {
			n.Parent.Children = append(siblings[:i:i], siblings[i+1:]...)
			break
		}
	}
	n.Parent = nil
}

// LastChild returns the last child or nil.
func (n *Node) LastChild() *Node {
	if len(n.Children) == 0 {
		return nil
	}
	return n.Children[len(n.Children)-1]
}

// Walk visits n and all its descendants in document order. Returning false
// from fn skips the node's children.
func (n *Node) Walk(fn func(*Node) bool) {
	if !fn(n) {
		return
	}
	for _, c := range n.Children {
		c.Walk(fn)
	}
}

// Elements returns every descendant element whose tag name satisfies match, in
// document order.
func (n *Node) Elements(match func(name string) bool) []*Node {
	var out []*Node
	n.Walk(func(node *Node) bool {
		if node.Kind == KindElement && match(node.El.Name) {
			out = append(out, node)
		}
		return true
	})
	return out
}

// TextContent concatenates every descendant text node, matching text_in_node in
// crates/mdbook-html/src/html/tree.rs.
func (n *Node) TextContent() string {
	var out []byte
	n.Walk(func(node *Node) bool {
		if node.Kind == KindText {
			out = append(out, node.Text...)
		}
		return true
	})
	return string(out)
}

// isVoidElement reports whether the element has no end tag. Ported from
// is_void_element in crates/mdbook-html/src/html/tree.rs.
func isVoidElement(name string) bool {
	switch name {
	case "area", "base", "br", "col", "embed", "hr", "img",
		"input", "link", "meta", "param", "source", "track", "wbr":
		return true
	}
	return false
}
