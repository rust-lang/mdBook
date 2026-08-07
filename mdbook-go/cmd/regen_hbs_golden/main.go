package main

import (
	"fmt"
	"os"
	"path/filepath"

	"mdbook-go/internal/hbs"
)

func main() {
	src, _ := os.ReadFile(filepath.Join("theme", "templates", "index.hbs"))
	head, _ := os.ReadFile(filepath.Join("theme", "templates", "head.hbs"))
	header, _ := os.ReadFile(filepath.Join("theme", "templates", "header.hbs"))
	r := hbs.New()
	_ = r.RegisterPartial("head", string(head))
	_ = r.RegisterPartial("header", string(header))
	_ = r.RegisterTemplate("index", string(src))
	resources := map[string]string{
		"favicon.svg": "favicon-de23e50b.svg", "favicon.png": "favicon-8114d1fc.png",
		"css/variables.css": "css/variables-8adf115d.css", "css/general.css": "css/general-e96d0476.css",
		"css/chrome.css": "css/chrome-d279d366.css", "fonts/fonts.css": "fonts/fonts-9644e21d.css",
		"highlight.css": "highlight-493f70e1.css", "tomorrow-night.css": "tomorrow-night-4c0ae647.css",
		"ayu-highlight.css": "ayu-highlight-3fdfc3ac.css", "searchindex.js": "searchindex-28707d4c.js",
		"toc.js": "toc-19c1b44c.js", "elasticlunr.min.js": "elasticlunr-ef4e11c1.min.js",
		"mark.min.js": "mark-09e88c2c.min.js", "searcher.js": "searcher-09f2665d.js",
		"clipboard.min.js": "clipboard-1626706a.min.js", "highlight.js": "highlight-abc7f01d.js",
		"book.js": "book-609e4cb8.js",
	}
	r.RegisterHelper("resource", func(_ *hbs.Context, p []any) (string, error) {
		name := p[0].(string)
		hashed, ok := resources[name]
		if !ok { return "", fmt.Errorf("unknown resource %q", name) }
		return hashed, nil
	})
	data := map[string]any{
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
		"path": "intro.md",
		"content": "<h1 id=\"introduction\"><a class=\"header\" href=\"#introduction\">Introduction</a></h1>\n<p>Welcome to the <strong>basic fixture</strong>. This is used to verify that mdbook-go can\nload a book and render chapters.</p>\n<ul>\n<li>Item one</li>\n<li>Item two</li>\n</ul>\n",
		"chapter_title": "Introduction", "title": "Introduction - Basic Fixture", "path_to_root": "",
		"next": map[string]any{"title": "Chapter 1", "link": "chapter_1.html"},
	}
	out, err := r.Render("index", data)
	if err != nil { fmt.Println(err); os.Exit(1) }
	if err := os.WriteFile("internal/hbs/testdata/intro.html", []byte(out), 0o644); err != nil { fmt.Println(err); os.Exit(1) }
	fmt.Println("regen ok, bytes:", len(out))
}
