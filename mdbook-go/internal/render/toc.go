package render

import "mdbook-go/internal/tplgotpl"

// As of 2026-08-06 the production sidebar is built by
// internal/tplgotpl/toc_render.go:renderTocSidebar. This file is kept only
// to expose the Env-side helpers that tgotpl expects; the legacy
// tocHelper (which used the hbs block-helper signature) is no longer
// referenced by the renderer.
//
// The Env struct is created per-page and passed into the .Resource / .TocHTML
// helper closures registered on the tplgotpl.Registry. See render.go's
// buildTocJSData and the tplgotpl.Env methods for the actual wiring.

// NewSidebarEnv returns a tplgotpl.Env configured for the no-JS toc.html iframe
// fallback. It is the production replacement for what was previously the
// {{#toc}} block helper invocation.
func NewSidebarEnv(chapters []any, foldEnable bool, foldLevel int, noSectionLabel bool) *tplgotpl.Env {
	return &tplgotpl.Env{
		Chapters:       chapters,
		FoldEnable:     foldEnable,
		FoldLevel:      foldLevel,
		NoSectionLabel: noSectionLabel,
	}
}
