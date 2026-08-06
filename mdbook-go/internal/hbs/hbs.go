// Package hbs implements the small Handlebars subset used by mdBook's theme.
//
// DEPRECATED as of 2026-08-06: the production renderer was switched to
// internal/tplgotpl (a thin html/template wrapper that mirrors this engine's
// surface). The package is kept in source for two reasons:
//
//  1. Rollback. If the tgotpl path turns out to be unworkable in production,
//     the render package can be reverted to this engine in a single change
//     (replace tgotpl.Registry with hbs.Registry in render/render.go,
//     print.go, and toc.go).
//  2. Regression coverage. internal/hbs/hbs_golden_test.go runs a
//     byte-exact golden test against the original mdBook hbs golden, which
//     is still considered the authoritative rendering of the canonical theme.
//
// No production code path imports this package after the 2026-08-06 switch.
package hbs

import (
	"fmt"
	"reflect"
	"strconv"
	"strings"
)

type Value any

type Helper func(ctx *Context, params []any) (string, error)
type BlockHelper func(ctx *Context, params []any, body func(data map[string]any) (string, error)) (string, error)

type Registry struct {
	templates    map[string]*template
	partials     map[string]*template
	helpers      map[string]Helper
	blockHelpers map[string]BlockHelper
}

type template struct{ nodes []node }

type node interface {
	render(*renderState) (string, error)
}

type textNode string

type exprNode struct {
	expr   expression
	escape bool
}

type partialNode struct {
	name   string
	indent string
}

type blockNode struct {
	name    string
	params  []argument
	body    []node
	inverse []node
}

type expression struct {
	name   string
	params []argument
}

type argument struct {
	literal bool
	value   any
	path    string
	sub     *expression
}

type frame struct {
	value  any
	parent *frame
}

type Context struct {
	root  map[string]any
	frame *frame
}

type renderState struct {
	reg *Registry
	ctx *Context
}

func New() *Registry {
	return &Registry{
		templates:    make(map[string]*template),
		partials:     make(map[string]*template),
		helpers:      make(map[string]Helper),
		blockHelpers: make(map[string]BlockHelper),
	}
}

func (r *Registry) RegisterTemplate(name, src string) error {
	t, err := parse(src, false)
	if err != nil {
		return fmt.Errorf("template %q: %w", name, err)
	}
	r.templates[name] = t
	return nil
}

func (r *Registry) RegisterPartial(name, src string) error {
	t, err := parse(src, true)
	if err != nil {
		return fmt.Errorf("partial %q: %w", name, err)
	}
	r.partials[name] = t
	return nil
}

func (r *Registry) RegisterHelper(name string, fn Helper) { r.helpers[name] = fn }
func (r *Registry) RegisterBlockHelper(name string, fn BlockHelper) {
	r.blockHelpers[name] = fn
}

func (r *Registry) Render(name string, data map[string]any) (string, error) {
	t, ok := r.templates[name]
	if !ok {
		return "", fmt.Errorf("template %q not registered", name)
	}
	ctx := &Context{root: data, frame: &frame{value: data}}
	return renderNodes(t.nodes, &renderState{reg: r, ctx: ctx})
}

// Lookup resolves a path relative to the current frame.
func (c *Context) Lookup(path string) (any, bool) {
	f := c.frame
	if path == "this" || path == "." {
		return f.value, true
	}
	if strings.HasPrefix(path, "@root") {
		path = strings.TrimPrefix(path, "@root")
		path = strings.TrimLeft(path, "/.")
		if path == "" {
			return c.root, true
		}
		return descend(c.root, splitPath(path))
	}
	for strings.HasPrefix(path, "../") {
		if f.parent != nil {
			f = f.parent
		}
		path = strings.TrimPrefix(path, "../")
	}
	path = strings.TrimPrefix(path, "./")
	if path == "this" || path == "" {
		return f.value, true
	}
	return descend(f.value, splitPath(path))
}

func splitPath(path string) []string {
	return strings.FieldsFunc(path, func(r rune) bool { return r == '/' || r == '.' })
}

func descend(v any, parts []string) (any, bool) {
	for _, p := range parts {
		switch x := v.(type) {
		case map[string]any:
			var ok bool
			v, ok = x[p]
			if !ok {
				return nil, false
			}
		case map[string]Value:
			var ok bool
			v, ok = x[p]
			if !ok {
				return nil, false
			}
		default:
			rv := reflect.ValueOf(v)
			if rv.Kind() == reflect.Map && rv.Type().Key().Kind() == reflect.String {
				mv := rv.MapIndex(reflect.ValueOf(p).Convert(rv.Type().Key()))
				if !mv.IsValid() {
					return nil, false
				}
				v = mv.Interface()
				continue
			}
			return nil, false
		}
	}
	return v, true
}

func parse(src string, isPartial bool) (*template, error) {
	toks, err := tokenize(src, isPartial)
	if err != nil {
		return nil, err
	}
	nodes, pos, stop, err := parseNodes(toks, 0, "")
	if err != nil {
		return nil, err
	}
	if pos != len(toks) || stop != "" {
		return nil, fmt.Errorf("unexpected closing tag %q", stop)
	}
	return &template{nodes: nodes}, nil
}

type token struct {
	text   string
	tag    string
	triple bool
	indent string
}

func tokenize(src string, isPartial bool) ([]token, error) {
	var out []token
	// prevStandalone records whether the statement just emitted consumed its
	// own line, which is what leaves the cursor at the start of a new line.
	prevStandalone := false
	for len(src) > 0 {
		i := strings.Index(src, "{{")
		if i < 0 {
			out = appendText(out, src)
			break
		}
		if i > 0 {
			out = appendText(out, src[:i])
		}
		src = src[i:]
		triple := strings.HasPrefix(src, "{{{") && !strings.HasPrefix(src, "{{!--")
		close := "}}"
		openLen := 2
		if triple {
			close, openLen = "}}}", 3
		}
		j := strings.Index(src[openLen:], close)
		if j < 0 {
			return nil, fmt.Errorf("unclosed expression")
		}
		j += openLen
		raw := src[:j+len(close)]
		tag := strings.TrimSpace(src[openLen:j])
		if strings.HasPrefix(raw, "{{!--") {
			k := strings.Index(src, "--}}")
			if k < 0 {
				return nil, fmt.Errorf("unclosed comment")
			}
			raw = src[:k+4]
			tag = "!"
		}
		src = src[len(raw):]

		standaloneKind := tag == "!" || strings.HasPrefix(tag, "#") || strings.HasPrefix(tag, "/") || tag == "else" || strings.HasPrefix(tag, ">")
		indent := ""
		isStandalone := false
		if standaloneKind {
			// A statement only counts as standalone when nothing but
			// whitespace precedes it on its line. If the previous token was
			// another statement, that is only true when that statement was
			// itself stripped, which is what put us back at the line start.
			linePrefix := ""
			leadingOK := true
			if len(out) > 0 {
				if last := out[len(out)-1]; last.tag == "" {
					linePrefix = trailingLine(out)
					leadingOK = strings.Trim(linePrefix, " \t") == ""
				} else {
					leadingOK = prevStandalone
				}
			}
			trimmed := strings.TrimLeft(src, " \t")
			newlineLen := 0
			if strings.HasPrefix(trimmed, "\r\n") {
				newlineLen = 2
			} else if strings.HasPrefix(trimmed, "\n") {
				newlineLen = 1
			} else if trimmed == "" && !isPartial {
				newlineLen = 0
			} else {
				leadingOK = false
			}
			if leadingOK {
				indent = linePrefix
				trimTrailingLine(&out)
				src = trimmed[newlineLen:]
				isStandalone = true
			}
		}
		prevStandalone = isStandalone
		if tag != "!" {
			out = append(out, token{tag: tag, triple: triple, indent: indent})
		}
	}
	return out, nil
}

func appendText(out []token, s string) []token {
	if s == "" {
		return out
	}
	if len(out) > 0 && out[len(out)-1].tag == "" {
		out[len(out)-1].text += s
	} else {
		out = append(out, token{text: s})
	}
	return out
}

func trailingLine(out []token) string {
	if len(out) == 0 || out[len(out)-1].tag != "" {
		return ""
	}
	s := out[len(out)-1].text
	if i := strings.LastIndexByte(s, '\n'); i >= 0 {
		return s[i+1:]
	}
	return s
}

func trimTrailingLine(out *[]token) {
	if len(*out) == 0 || (*out)[len(*out)-1].tag != "" {
		return
	}
	s := (*out)[len(*out)-1].text
	if i := strings.LastIndexByte(s, '\n'); i >= 0 {
		s = s[:i+1]
	} else {
		s = ""
	}
	if s == "" {
		*out = (*out)[:len(*out)-1]
	} else {
		(*out)[len(*out)-1].text = s
	}
}

func parseNodes(toks []token, pos int, close string) ([]node, int, string, error) {
	var nodes []node
	for pos < len(toks) {
		t := toks[pos]
		if t.tag == "" {
			nodes = append(nodes, textNode(t.text))
			pos++
			continue
		}
		if t.tag == "else" || strings.HasPrefix(t.tag, "/") {
			return nodes, pos, t.tag, nil
		}
		if strings.HasPrefix(t.tag, "#") {
			expr, err := parseExpression(strings.TrimSpace(t.tag[1:]))
			if err != nil {
				return nil, pos, "", err
			}
			body, next, stop, err := parseNodes(toks, pos+1, expr.name)
			if err != nil {
				return nil, pos, "", err
			}
			var inverse []node
			if stop == "else" {
				inverse, next, stop, err = parseNodes(toks, next+1, expr.name)
				if err != nil {
					return nil, pos, "", err
				}
			}
			if stop != "/"+expr.name {
				return nil, pos, "", fmt.Errorf("block %q not closed", expr.name)
			}
			nodes = append(nodes, blockNode{name: expr.name, params: expr.params, body: body, inverse: inverse})
			pos = next + 1
			continue
		}
		if strings.HasPrefix(t.tag, ">") {
			nodes = append(nodes, partialNode{name: strings.TrimSpace(t.tag[1:]), indent: t.indent})
			pos++
			continue
		}
		expr, err := parseExpression(t.tag)
		if err != nil {
			return nil, pos, "", err
		}
		nodes = append(nodes, exprNode{expr: expr, escape: !t.triple})
		pos++
	}
	if close != "" {
		return nodes, pos, "", fmt.Errorf("block %q not closed", close)
	}
	return nodes, pos, "", nil
}

func parseExpression(s string) (expression, error) {
	parts, err := fields(s)
	if err != nil || len(parts) == 0 {
		return expression{}, fmt.Errorf("invalid expression %q", s)
	}
	e := expression{name: parts[0]}
	for _, p := range parts[1:] {
		a, err := parseArgument(p)
		if err != nil {
			return expression{}, err
		}
		e.params = append(e.params, a)
	}
	return e, nil
}

func fields(s string) ([]string, error) {
	var result []string
	for i := 0; i < len(s); {
		for i < len(s) && (s[i] == ' ' || s[i] == '\t' || s[i] == '\n' || s[i] == '\r') {
			i++
		}
		if i == len(s) {
			break
		}
		start := i
		switch s[i] {
		case '\'', '"':
			q := s[i]
			i++
			for i < len(s) && s[i] != q {
				if s[i] == '\\' {
					i++
				}
				i++
			}
			if i == len(s) {
				return nil, fmt.Errorf("unterminated string")
			}
			i++
		case '(':
			depth := 1
			i++
			for i < len(s) && depth > 0 {
				if s[i] == '(' {
					depth++
				} else if s[i] == ')' {
					depth--
				}
				i++
			}
			if depth != 0 {
				return nil, fmt.Errorf("unterminated subexpression")
			}
		default:
			for i < len(s) && !strings.ContainsRune(" \t\r\n", rune(s[i])) {
				i++
			}
		}
		result = append(result, s[start:i])
	}
	return result, nil
}

func parseArgument(s string) (argument, error) {
	if len(s) >= 2 && (s[0] == '"' || s[0] == '\'') {
		q := s[0]
		body := s[1 : len(s)-1]
		if q == '\'' {
			body = strings.ReplaceAll(body, `\'`, `'`)
			return argument{literal: true, value: body}, nil
		}
		v, err := strconv.Unquote(s)
		return argument{literal: true, value: v}, err
	}
	if strings.HasPrefix(s, "(") && strings.HasSuffix(s, ")") {
		e, err := parseExpression(strings.TrimSpace(s[1 : len(s)-1]))
		return argument{sub: &e}, err
	}
	switch s {
	case "true":
		return argument{literal: true, value: true}, nil
	case "false":
		return argument{literal: true, value: false}, nil
	case "null":
		return argument{literal: true, value: nil}, nil
	}
	if n, err := strconv.ParseFloat(s, 64); err == nil {
		return argument{literal: true, value: n}, nil
	}
	return argument{path: s}, nil
}

func (n textNode) render(*renderState) (string, error) { return string(n), nil }

func (n exprNode) render(s *renderState) (string, error) {
	v, err := evalExpression(n.expr, s)
	if err != nil {
		return "", err
	}
	out := stringify(v)
	// Helpers write straight to the output stream in handlebars-rust
	// (`out.write(...)`), so their result is never HTML-escaped, even in a
	// double-brace expression. Only plain path lookups are escaped.
	if _, raw := v.(rawString); n.escape && !raw {
		out = escapeHTML(out)
	}
	return out, nil
}

func (n partialNode) render(s *renderState) (string, error) {
	t, ok := s.reg.partials[n.name]
	if !ok {
		return "", fmt.Errorf("partial %q not registered", n.name)
	}
	out, err := renderNodes(t.nodes, s)
	if err != nil || n.indent == "" {
		return out, err
	}
	return strings.ReplaceAll(strings.TrimSuffix(out, "\n"), "\n", "\n"+n.indent) + suffixNewline(out), nil
}

func suffixNewline(s string) string {
	if strings.HasSuffix(s, "\n") {
		return "\n"
	}
	return ""
}

func (n blockNode) render(s *renderState) (string, error) {
	params, err := evalArguments(n.params, s)
	if err != nil {
		return "", err
	}
	if fn, ok := s.reg.blockHelpers[n.name]; ok {
		body := func(data map[string]any) (string, error) {
			ctx := &Context{root: s.ctx.root, frame: &frame{value: data, parent: s.ctx.frame}}
			return renderNodes(n.body, &renderState{reg: s.reg, ctx: ctx})
		}
		return fn(s.ctx, params, body)
	}
	switch n.name {
	case "if":
		if len(params) != 1 {
			return "", fmt.Errorf("if expects one parameter")
		}
		if truthy(params[0]) {
			return renderNodes(n.body, s)
		}
		return renderNodes(n.inverse, s)
	case "each":
		if len(params) != 1 {
			return "", fmt.Errorf("each expects one parameter")
		}
		rv := reflect.ValueOf(params[0])
		if !rv.IsValid() || (rv.Kind() != reflect.Slice && rv.Kind() != reflect.Array) {
			return renderNodes(n.inverse, s)
		}
		var b strings.Builder
		for i := 0; i < rv.Len(); i++ {
			ctx := &Context{root: s.ctx.root, frame: &frame{value: rv.Index(i).Interface(), parent: s.ctx.frame}}
			part, err := renderNodes(n.body, &renderState{reg: s.reg, ctx: ctx})
			if err != nil {
				return "", err
			}
			b.WriteString(part)
		}
		return b.String(), nil
	default:
		return "", fmt.Errorf("block helper %q not registered", n.name)
	}
}

func renderNodes(nodes []node, s *renderState) (string, error) {
	var b strings.Builder
	for _, n := range nodes {
		part, err := n.render(s)
		if err != nil {
			return "", err
		}
		b.WriteString(part)
	}
	return b.String(), nil
}

func evalExpression(e expression, s *renderState) (any, error) {
	if len(e.params) == 0 {
		if v, ok := s.ctx.Lookup(e.name); ok {
			return v, nil
		}
	}
	params, err := evalArguments(e.params, s)
	if err != nil {
		return nil, err
	}
	if e.name == "eq" {
		if len(params) != 2 {
			return nil, fmt.Errorf("eq expects two parameters")
		}
		return reflect.DeepEqual(params[0], params[1]), nil
	}
	fn, ok := s.reg.helpers[e.name]
	if !ok {
		if len(e.params) == 0 {
			return nil, nil
		}
		return nil, fmt.Errorf("helper %q not registered", e.name)
	}
	out, err := fn(s.ctx, params)
	if err != nil {
		return nil, err
	}
	return rawString(out), nil
}

// rawString marks a value that must be emitted verbatim. Helper output is
// written directly to the stream by handlebars-rust and so is never escaped.
type rawString string

func evalArguments(args []argument, s *renderState) ([]any, error) {
	out := make([]any, len(args))
	for i, a := range args {
		if a.literal {
			out[i] = a.value
		} else if a.sub != nil {
			v, err := evalExpression(*a.sub, s)
			if err != nil {
				return nil, err
			}
			out[i] = v
		} else {
			out[i], _ = s.ctx.Lookup(a.path)
		}
	}
	return out, nil
}

func truthy(v any) bool {
	if v == nil {
		return false
	}
	rv := reflect.ValueOf(v)
	switch rv.Kind() {
	case reflect.Bool:
		return rv.Bool()
	case reflect.String, reflect.Array, reflect.Slice, reflect.Map:
		return rv.Len() != 0
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return rv.Int() != 0
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return rv.Uint() != 0
	case reflect.Float32, reflect.Float64:
		return rv.Float() != 0
	}
	return true
}

func stringify(v any) string {
	if v == nil {
		return ""
	}
	switch x := v.(type) {
	case string:
		return x
	case bool:
		return strconv.FormatBool(x)
	case fmt.Stringer:
		return x.String()
	default:
		return fmt.Sprint(x)
	}
}

func escapeHTML(s string) string {
	var b strings.Builder
	for _, r := range s {
		switch r {
		case '<':
			b.WriteString("&lt;")
		case '>':
			b.WriteString("&gt;")
		case '"':
			b.WriteString("&quot;")
		case '&':
			b.WriteString("&amp;")
		case '\'':
			b.WriteString("&#x27;")
		case '`':
			b.WriteString("&#x60;")
		case '=':
			b.WriteString("&#x3D;")
		default:
			b.WriteRune(r)
		}
	}
	return b.String()
}
