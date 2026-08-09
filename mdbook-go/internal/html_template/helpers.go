package html_template

import (
	"html/template"
	"path/filepath"
	"strings"
)

// Env carries the per-build state that the template helpers need. It mirrors
// the closures built in render.go: resources are only known after static
// files are written, and the `path` of the current page influences how many
// `../` the `resource` helper emits.
type Env struct {
	// Resources maps a logical asset name (e.g. "css/chrome.css") to the
	// hashed filename in the output directory (e.g. "chrome-d279d366.css").
	Resources map[string]string
	// Path is the current page path inside the book (e.g. "toc.html" or
	// "chapter_1.html"). Used by Resource to compute `../` prefixes.
	Path string
	// Chapters is the flattened chapter tree. Consumed by TocHTML.
	Chapters []any
	// FoldEnable / FoldLevel mirror [output.html.fold] settings; used by
	// TocHTML to decide which sections render expanded.
	FoldEnable bool
	FoldLevel  int
	// IsTocHTML tells TocHTML whether it is rendering the no-JS fallback
	// (toc.html iframe) or the JS-embedded sidebar (toc.js script).
	IsTocHTML bool
	// NoSectionLabel mirrors config.HtmlConfig.NoSectionLabel.
	NoSectionLabel bool
	// SidebarHeaderNav mirrors config.HtmlConfig.SidebarHeaderNav. Used by
	// RenderTocJS to decide whether to splice the header-tracking IIFE.
	SidebarHeaderNav bool
	// SidebarHeaderNavBlocks is the literal JS block that implements
	// header tracking. RenderTocJS decides whether to emit it based on
	// SidebarHeaderNav.
	SidebarHeaderNavBlocks string
	// Content is the rendered chapter HTML; passed as template.HTML so
	// html/template does not re-escape it.
	Content template.HTML
	// LiveReloadEndpoint is the URL fragment the live-reload WebSocket
	// connects to; passed as template.URL.
	LiveReloadEndpoint template.URL
	// FragmentMap is the redirect-fragment map for this page; passed as
	// template.JS so it survives the JS string literal in index.html
	// unescaped.
	FragmentMap template.JS
}

// Resource implements `{{resource "name"}}`. Emits the asset path with enough
// `../` to reach the output root from the current page.
func (e *Env) Resource(name string) string {
	resolved, ok := e.Resources[name]
	if !ok {
		resolved = name
	}
	return pathToRoot(e.Path) + resolved
}

// TocHTML returns the sidebar HTML for use inside a template via
// `{{.TocHTML}}`. Pre-computing this lets us avoid the block-helper shape
// `{{#toc}}…{{/toc}}`, which html/template does not support.
func (e *Env) TocHTML() template.HTML {
	return template.HTML(renderTocSidebar(e.Chapters, e.FoldEnable, e.FoldLevel, e.NoSectionLabel, e.IsTocHTML))
}

// SidebarHeaderNavJS returns the literal JS block that the old
// `{{#if sidebar_header_nav}}…{{/if}}` template emitted. We pre-stash it via
// Env.SidebarHeaderNavBlocks and emit it as a single {{.X}} interpolation.
//
// Returns empty string when the feature is disabled, so the calling code can
// use `{{if .SidebarHeaderNavJS}}{{.SidebarHeaderNavJS}}{{end}}`.
func (e *Env) SidebarHeaderNavJS() template.JS {
	if !e.SidebarHeaderNav {
		return ""
	}
	return template.JS(e.SidebarHeaderNavBlocks)
}

// pathToRoot returns the relative prefix needed to reach the output root from
// the given page path. Mirrors fs.PathToRoot.
func pathToRoot(p string) string {
	if p == "" {
		return ""
	}
	dir := filepath.ToSlash(filepath.Dir(p))
	if dir == "." || dir == "" {
		return ""
	}
	depth := strings.Count(dir, "/") + 1
	return strings.Repeat("../", depth)
}
