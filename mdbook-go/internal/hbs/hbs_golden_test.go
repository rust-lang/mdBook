package hbs

import (
	"os"
	"path/filepath"
	"regexp"

	"testing"
)

func themeFile(t *testing.T, name string) string {
	t.Helper()
	b, err := os.ReadFile(filepath.Join("..", "..", "theme", "templates", name))
	if err != nil {
		t.Fatal(err)
	}
	return string(b)
}

func golden(t *testing.T, name string) string {
	t.Helper()
	b, err := os.ReadFile(filepath.Join("testdata", name))
	if err != nil {
		t.Fatal(err)
	}
	return string(b)
}

func basicData() map[string]any {
	return map[string]any{
		"language": "en", "text_direction": "ltr", "book_title": "Basic Fixture",
		"description": "", "favicon_png": "favicon.png", "favicon_svg": "favicon.svg",
		"default_theme": "light", "preferred_dark_theme": "navy", "print_enable": true,
		"fold_enable": false, "fold_level": 0, "search_enabled": true, "search_js": true,
		"playground_copyable": true, "sidebar_header_nav": false,
		"chapters": []any{
			map[string]any{"section": "", "has_sub_items": "false", "name": "Introduction", "path": "intro.md"},
			map[string]any{"part": "Chapter 1"},
			map[string]any{"section": "1.", "has_sub_items": "true", "name": "Chapter 1", "path": "chapter_1.md"},
			map[string]any{"section": "1.1.", "has_sub_items": "false", "name": "Section 1.1", "path": "section_1_1.md"},
			map[string]any{"section": "2.", "has_sub_items": "false", "name": "Chapter 2", "path": "chapter_2.md"},
			map[string]any{"part": "Drafts"},
			map[string]any{"section": "3.", "has_sub_items": "false", "name": "Unwritten"},
		},
		"path": "intro.md", "content": `<h1 id="introduction"><a class="header" href="#introduction">Introduction</a></h1>
<p>Welcome to the <strong>basic fixture</strong>. This is used to verify that mdbook-go can
load a book and render chapters.</p>
<ul>
<li>Item one</li>
<li>Item two</li>
</ul>
`,
		"chapter_title": "Introduction", "title": "Introduction - Basic Fixture", "path_to_root": "",
		"next": map[string]any{"title": "Chapter 1", "link": "chapter_1.html"},
	}
}

func registry(t *testing.T, template, source string) *Registry {
	t.Helper()
	r := New()
	if err := r.RegisterTemplate(template, source); err != nil {
		t.Fatal(err)
	}
	if err := r.RegisterPartial("head", themeFile(t, "head.hbs")); err != nil {
		t.Fatal(err)
	}
	if err := r.RegisterPartial("header", themeFile(t, "header.hbs")); err != nil {
		t.Fatal(err)
	}
	return r
}

func TestIndexGolden(t *testing.T) {
	want := golden(t, "intro.html")
	r := registry(t, "index", themeFile(t, "index.hbs"))
	// The hashed names the Rust build produced for this fixture. Transcribed
	// from the golden output rather than recomputed, so the test exercises the
	// template engine and nothing else.
	resources := map[string]string{
		"favicon.svg":        "favicon-de23e50b.svg",
		"favicon.png":        "favicon-8114d1fc.png",
		"css/variables.css":  "css/variables-8adf115d.css",
		"css/general.css":    "css/general-e96d0476.css",
		"css/chrome.css":     "css/chrome-d279d366.css",
		"css/print.css":      "css/print-9e4910d8.css",
		"fonts/fonts.css":    "fonts/fonts-9644e21d.css",
		"highlight.css":      "highlight-493f70e1.css",
		"tomorrow-night.css": "tomorrow-night-4c0ae647.css",
		"ayu-highlight.css":  "ayu-highlight-3fdfc3ac.css",
		"searchindex.js":     "searchindex-28707d4c.js",
		"toc.js":             "toc-19c1b44c.js",
		"elasticlunr.min.js": "elasticlunr-ef4e11c1.min.js",
		"mark.min.js":        "mark-09e88c2c.min.js",
		"searcher.js":        "searcher-09f2665d.js",
		"clipboard.min.js":   "clipboard-1626706a.min.js",
		"highlight.js":       "highlight-abc7f01d.js",
		"book.js":            "book-609e4cb8.js",
	}
	r.RegisterHelper("resource", func(_ *Context, p []any) (string, error) {
		name := p[0].(string)
		hashed, ok := resources[name]
		if !ok {
			t.Fatalf("template asked for unknown resource %q", name)
		}
		return hashed, nil
	})
	faRE := regexp.MustCompile(`<span class=fa-svg(?: id="([^"]+)")?><svg[^\n]*?</svg></span>`)
	icons := faRE.FindAllString(want, -1)
	calls := 0
	r.RegisterHelper("fa", func(_ *Context, _ []any) (string, error) {
		out := icons[calls]
		calls++
		return out, nil
	})
	got, err := r.Render("index", basicData())
	if err != nil {
		t.Fatal(err)
	}
	if got != want {
		i := firstDiff(got, want)
		lo, hi := max(0, i-80), min(len(got), i+160)
		wlo, whi := max(0, i-80), min(len(want), i+160)
		t.Fatalf("index output differs at byte %d (got %d bytes, want %d)\ngot: %q\nwant:%q", i, len(got), len(want), got[lo:hi], want[wlo:whi])
	}
}

func TestTOCGolden(t *testing.T) {
	want := golden(t, "toc.html")
	r := registry(t, "toc", themeFile(t, "toc.html.hbs"))
	resources := map[string]string{
		"css/variables.css": "css/variables-8adf115d.css", "css/general.css": "css/general-e96d0476.css",
		"css/chrome.css": "css/chrome-d279d366.css", "css/print.css": "css/print-9e4910d8.css",
		"fonts/fonts.css": "fonts/fonts-9644e21d.css",
	}
	r.RegisterHelper("resource", func(_ *Context, p []any) (string, error) { return resources[p[0].(string)], nil })
	toc := regexp.MustCompile(`<ol class="chapter">.*</ol>`).FindString(want)
	r.RegisterBlockHelper("toc", func(_ *Context, _ []any, _ func(map[string]any) (string, error)) (string, error) {
		return toc, nil
	})
	got, err := r.Render("toc", basicData())
	if err != nil {
		t.Fatal(err)
	}
	if got != want {
		i := firstDiff(got, want)
		lo, hi := max(0, i-80), min(len(got), i+160)
		wlo, whi := max(0, i-80), min(len(want), i+160)
		t.Fatalf("toc output differs at byte %d (got %d bytes, want %d)\ngot: %q\nwant:%q", i, len(got), len(want), got[lo:hi], want[wlo:whi])
	}
}

func firstDiff(a, b string) int {
	for i := 0; i < len(a) && i < len(b); i++ {
		if a[i] != b[i] {
			return i
		}
	}
	if len(a) != len(b) {
		if len(a) < len(b) {
			return len(a)
		}
		return len(b)
	}
	return -1
}
