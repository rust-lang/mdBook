// Package tplgotpl is a Go-template port of mdbook-go's Handlebars theme.
//
// As of 2026-08-06 this package is the production renderer for mdbook-go.
// The parallel hbs engine (internal/hbs) is kept for rollback and for the
// hbs_golden_test byte-level regression suite, but no production code path
// imports it anymore.
//
// Public surface:
//   - Registry:     mirrors hbs.Registry (New / RegisterTemplate /
//                   RegisterPartial / RegisterFunc / Render / LoadTemplates).
//   - Env:          per-build state threaded into template helpers.
//   - RenderTocJS:  builds the toc.js source as a single Go string, splicing
//                   in the sidebar HTML via template.JS. We do not template
//                   toc.js because its body contains {{#toc}}{{/toc}} inside
//                   a single-quoted JS string, which html/template cannot
//                   evaluate.
//
// The 5 .gohtml files in prod/ (index, redirect, toc.html, head, header) are
// translated 1:1 from theme/templates/*.hbs with these substitutions:
//
//   {{var}}        -> {{.Var}}
//   {{{raw}}}      -> {{.Raw}}        (with Raw typed template.HTML/JS/URL)
//   {{#if X}}      -> {{if .X}}
//   {{#each X}}    -> {{range .X}}
//   {{> partial}}  -> {{template "name" .}}
//   {{helper …}}   -> registered via Env's FuncMap
//   {{#toc}}       -> {{.TocHTML}}    (pre-computed by Env.TocHTML)
//   {{#if (eq A B)}} -> {{if eq .A .B}}
package tplgotpl

import (
	"bytes"
	"embed"
	"fmt"
	"html/template"
	"io/fs"
	"path"
	"strings"
	"sync"
)

//go:embed all:prod
var prodFS embed.FS

// prodFSRoot is the directory inside the embed where the production templates
// live. The Registry.LoadProduction helper uses this to wire every .gohtml in
// prod/ into a single Registry in one call.
const prodFSRoot = "prod"

// Registry wraps a set of html/template.Template instances keyed by name.
type Registry struct {
	mu        sync.Mutex
	funcs     template.FuncMap
	partials  map[string]string
	templates map[string]*template.Template
}

// New creates an empty Registry. Templates registered afterwards share the
// same func map and partial set; adding a helper after a template is parsed
// does not retroactively change that template (html/template semantics).
func New() *Registry {
	return &Registry{
		funcs:     template.FuncMap{},
		partials:  make(map[string]string),
		templates: make(map[string]*template.Template),
	}
}

// RegisterPartial stores a partial's source AND parses it eagerly so that any
// template parsed afterwards can attach it via AddParseTree. Partials become
// addressable as {{template "name" .}} inside any template loaded afterwards.
func (r *Registry) RegisterPartial(name, src string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.partials[name] = src
	t, err := template.New(name).Funcs(r.funcs).Parse(stripStandaloneLines(src))
	if err != nil {
		return fmt.Errorf("partial %s: %w", name, err)
	}
	r.templates["partial:"+name] = t
	return nil
}

// RegisterFunc adds a function to the FuncMap. Like html/template, calling
// RegisterFunc after a template has been parsed does not retroactively affect
// that template; call it before LoadTemplates.
func (r *Registry) RegisterFunc(name string, fn any) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.funcs[name] = fn
}

// LoadTemplates reads every *.gohtml file under root inside fsys and parses
// it as a top-level template keyed by its base name (without extension).
func (r *Registry) LoadTemplates(fsys fs.FS, root string) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	// Pre-parse every registered partial so we can attach its parse tree to
	// each template via AddParseTree.
	for name, src := range r.partials {
		t, err := template.New(name).Funcs(r.funcs).Parse(stripStandaloneLines(src))
		if err != nil {
			return fmt.Errorf("partial %s: %w", name, err)
		}
		r.templates["partial:"+name] = t
	}

	entries, err := fs.ReadDir(fsys, root)
	if err != nil {
		return err
	}
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".gohtml") {
			continue
		}
		raw, err := fs.ReadFile(fsys, path.Join(root, e.Name()))
		if err != nil {
			return err
		}
		name := strings.TrimSuffix(e.Name(), ".gohtml")
		if _, ok := r.templates[name]; ok {
			continue
		}
		if err := r.parseLocked(name, string(raw)); err != nil {
			return err
		}
	}
	return nil
}

// LoadTemplate parses a single named template from src. Convenient when the
// caller wants to register funcs first and then add one template at a time.
func (r *Registry) LoadTemplate(name, src string) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.parseLocked(name, src)
}

// RegisterTemplate stores src under name and parses it. Equivalent to
// LoadTemplate; kept for symmetry with the partial/helper API.
func (r *Registry) RegisterTemplate(name, src string) error {
	return r.LoadTemplate(name, src)
}

// LoadProduction parses the five .gohtml files in prod/ as top-level
// templates (index, redirect, toc.html, head, header). Call after registering
// helpers and partials on r.
func (r *Registry) LoadProduction() error {
	return r.LoadTemplates(prodFS, prodFSRoot)
}

// parseLocked parses src as a top-level template named name, attaching every
// previously-registered partial via AddParseTree.
func (r *Registry) parseLocked(name, src string) error {
	src = stripStandaloneLines(src)
	t := template.New(name).Funcs(r.funcs)
	for pname, pt := range r.templates {
		if !strings.HasPrefix(pname, "partial:") {
			continue
		}
		if _, err := t.AddParseTree(strings.TrimPrefix(pname, "partial:"), pt.Tree); err != nil {
			return fmt.Errorf("attach partial %s to %s: %w", pname, name, err)
		}
	}
	if _, err := t.Parse(src); err != nil {
		return fmt.Errorf("template %s: %w", name, err)
	}
	r.templates[name] = t
	return nil
}

// Render executes the named template against data and returns the output.
func (r *Registry) Render(name string, data any) (string, error) {
	r.mu.Lock()
	t, ok := r.templates[name]
	r.mu.Unlock()
	if !ok {
		return "", fmt.Errorf("template %q not registered", name)
	}
	var buf bytes.Buffer
	if err := t.Execute(&buf, data); err != nil {
		return "", err
	}
	return buf.String(), nil
}

// HasTemplate reports whether name has been registered (top-level or partial).
// Useful for callers that want to detect a missing template without parsing.
func (r *Registry) HasTemplate(name string) bool {
	r.mu.Lock()
	defer r.mu.Unlock()
	_, ok := r.templates[name]
	return ok
}
