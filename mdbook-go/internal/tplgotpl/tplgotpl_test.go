// Package tplgotpl_test runs the side-by-side comparison between the
// hbs engine (production) and the tplgotpl engine (Go template POC).
// Run with `go test ./internal/tplgotpl/ -v` to see the per-template diff.
//
// These tests are INFORMATIONAL: they never fail on byte-differences
// because html/template is known to diverge from raw-text hbs output in
// several places (see TestRedirect_Comparison and TestTocHTML_Comparison
// for the per-template delta). They only fail on hard errors
// (parse failure, panic, render error). The point is to surface the
// diff so a human can judge whether the divergence is acceptable.
package tplgotpl_test

import (
	"embed"
	"fmt"
	"html/template"
	"strings"
	"testing"

	"mdbook-go/internal/hbs"
	tgotpl "mdbook-go/internal/tplgotpl"
)

//go:embed testdata/hbs/*.hbs testdata/gotpl/*.gohtml
var fixturesFS embed.FS

// loadBoth loads the partial set on both engines and the named template on
// the hbs side. The caller is expected to register any required helpers
// on the tplgotpl side and then call RegisterTemplate themselves; that
// ordering is necessary because html/template refuses to resolve funcs
// that are not registered at parse time.
func loadBoth(t *testing.T, name string) (*hbs.Registry, *tgotpl.Registry) {
	t.Helper()

	hr := hbs.New()
	if err := hr.RegisterPartial("head", mustRead(t, fixturesFS, "testdata/hbs/head.hbs")); err != nil {
		t.Fatalf("hbs partial head: %v", err)
	}
	if err := hr.RegisterPartial("header", mustRead(t, fixturesFS, "testdata/hbs/header.hbs")); err != nil {
		t.Fatalf("hbs partial header: %v", err)
	}
	if err := hr.RegisterTemplate(name, mustRead(t, fixturesFS, "testdata/hbs/"+name+".hbs")); err != nil {
		t.Fatalf("hbs template %s: %v", name, err)
	}

	gr := tgotpl.New()
	if err := gr.RegisterPartial("head", mustRead(t, fixturesFS, "testdata/gotpl/head.gohtml")); err != nil {
		t.Fatalf("tplgotpl partial head: %v", err)
	}
	if err := gr.RegisterPartial("header", mustRead(t, fixturesFS, "testdata/gotpl/header.gohtml")); err != nil {
		t.Fatalf("tplgotpl partial header: %v", err)
	}
	return hr, gr
}

func mustRead(t *testing.T, fs embed.FS, p string) string {
	t.Helper()
	b, err := fs.ReadFile(p)
	if err != nil {
		t.Fatalf("read %s: %v", p, err)
	}
	return string(b)
}

// ---------- redirect ----------

func TestRedirect_Comparison(t *testing.T) {
	hr, gr := loadBoth(t, "redirect")
	if err := gr.RegisterTemplate("redirect", mustRead(t, fixturesFS, "testdata/gotpl/redirect.gohtml")); err != nil {
		t.Fatalf("tplgotpl template redirect: %v", err)
	}

	url := "https://example.com/landing-page/"
	frag := `{"#a":"/landing.html#a","#b":"/landing.html#b"}`

	hrOut, err := hr.Render("redirect", map[string]any{
		"url":          url,
		"fragment_map": frag,
	})
	if err != nil {
		t.Fatalf("hbs render: %v", err)
	}
	grOut, err := gr.Render("redirect", redirectData{
		URL:         template.URL(url),
		FragmentMap: template.JS(frag),
	})
	if err != nil {
		t.Fatalf("gotpl render: %v", err)
	}

	reportDiff(t, "redirect", hrOut, grOut)
}

type redirectData struct {
	URL         template.URL
	FragmentMap template.JS
}

// ---------- toc.html ----------

func TestTocHTML_Comparison(t *testing.T) {
	hr, gr := loadBoth(t, "toc.html")

	resources := map[string]string{
		"css/variables.css": "variables-8adf115d.css",
		"css/general.css":    "general-e96d0476.css",
		"css/chrome.css":     "chrome-d279d366.css",
		"css/print.css":      "print-9e4910d8.css",
		"fonts/fonts.css":    "fonts-9644e21d.css",
	}
	data := map[string]any{
		"language":         "en",
		"default_theme":    "light",
		"text_direction":   "ltr",
		"base_url":         "",
		"print_enable":     true,
		"additional_css":   []any{"theme-overrides.css"},
		"path":             "toc.html",
		"chapters":         sampleChapters(),
		"fold_enable":      false,
		"fold_level":       1,
		"is_toc_html":      true,
		"no_section_label": false,
	}

	hr.RegisterHelper("resource", resourceHelperHbs(resources))
	hr.RegisterBlockHelper("toc", tocBlockHbs)

	hrOut, err := hr.Render("toc.html", data)
	if err != nil {
		t.Fatalf("hbs render: %v", err)
	}

	env := &tgotpl.Env{
		Resources:      resources,
		Path:           "toc.html",
		Chapters:       sampleChapters(),
		FoldEnable:     false,
		FoldLevel:      1,
		IsTocHTML:      true,
		NoSectionLabel: false,
	}
	gr.RegisterFunc("resource", env.Resource)
	gr.RegisterFunc("tocHTML", env.TocHTML)

	if err := gr.RegisterTemplate("toc.html", mustRead(t, fixturesFS, "testdata/gotpl/toc.html.gohtml")); err != nil {
		t.Fatalf("tplgotpl template toc.html: %v", err)
	}

	// toc.html.gohtml uses .Language / .DefaultTheme / etc. — Go template
	// uppercases the first letter on map lookup, so wrap the map in a
	// struct with exported fields for parity.
	grOut, err := gr.Render("toc.html", tocData{
		Language:      "en",
		DefaultTheme:  "light",
		TextDirection: "ltr",
		BaseURL:       "",
		PrintEnable:   true,
		AdditionalCSS: []string{"theme-overrides.css"},
	})
	if err != nil {
		t.Fatalf("gotpl render: %v", err)
	}

	if hrOut != grOut {
		reportDiff(t, "toc.html", hrOut, grOut)
	} else {
		t.Logf("toc.html: byte-identical (%d bytes)", len(hrOut))
	}
}

type tocData struct {
	Language      string
	DefaultTheme  string
	TextDirection string
	BaseURL       string
	PrintEnable   bool
	AdditionalCSS []string
}

// ---------- helpers ----------

func sampleChapters() []any {
	return []any{
		map[string]any{"name": "Introduction", "path": "intro.html"},
		map[string]any{"name": "Getting Started", "path": "getting-started.html"},
		map[string]any{"name": "Chapter 1", "section": "1", "path": "chapter_1.html", "has_sub_items": "true"},
		map[string]any{"name": "Sub 1.1", "section": "1.1", "path": "sub_1_1.html"},
		map[string]any{"name": "Sub 1.2", "section": "1.2", "path": "sub_1_2.html"},
		map[string]any{"name": "Conclusion", "path": "conclusion.html"},
	}
}

// resourceHelperHbs is a copy of render.go's resourceHelper, adapted to
// the test's data shape. Kept inline so the test does not depend on
// internal/render.
func resourceHelperHbs(resources map[string]string) hbs.Helper {
	return func(ctx *hbs.Context, params []any) (string, error) {
		if len(params) < 1 {
			return "", fmt.Errorf("resource helper expects a name")
		}
		name := fmt.Sprint(params[0])
		basePath := ""
		if v, ok := ctx.Lookup("@root/path"); ok {
			basePath = strings.Trim(fmt.Sprint(v), `"`)
		}
		resolved, ok := resources[name]
		if !ok {
			resolved = name
		}
		return pathToRootHbs(basePath) + resolved, nil
	}
}

// tocBlockHbs mirrors the production tocHelper but is hard-coded to
// emit a stub. The real test is whether the *engine* handles the
// block-helper invocation correctly; the *content* of the TOC is
// already validated by the existing render tests.
func tocBlockHbs(_ *hbs.Context, _ []any, _ func(map[string]any) (string, error)) (string, error) {
	return `<ol class="chapter"></ol>`, nil
}

// pathToRootHbs is the hbs-side mirror of utils.PathToRoot, kept local so
// the test does not depend on internal/utils.
func pathToRootHbs(p string) string {
	if p == "" {
		return ""
	}
	depth := strings.Count(p, "/")
	if strings.HasSuffix(p, "/index.html") || strings.HasSuffix(p, "/index.htm") {
		depth++
	}
	return strings.Repeat("../", depth)
}

// ---------- diff pretty-print ----------

// reportDiff logs the byte counts and unified diff between the two engine
// outputs and never fails the test. Categorising the divergences is a
// human task; the test only ensures both engines actually ran.
func reportDiff(t *testing.T, label, hrOut, grOut string) {
	t.Helper()
	t.Logf("=== %s ===", label)
	t.Logf("hbs   (%d bytes)", len(hrOut))
	t.Logf("gotpl (%d bytes)", len(grOut))
	if hrOut == grOut {
		t.Logf("byte-identical")
		return
	}
	diff := unifiedDiff(label+".hbs", hrOut, label+".gohtml", grOut)
	t.Logf("diff (first 100 lines):\n%s", trunc(diff, 100))

	// Surface the known divergence categories so the reader doesn't have
	// to scan the raw diff to see them.
	if strings.Contains(grOut, `\/`) || strings.Contains(grOut, `&`) {
		t.Logf("note: gotpl output contains backslash-escaped chars — html/template " +
			"escapes / and & inside JS string literals. This is the documented " +
			"Go template behaviour.")
	}
	if !containsAll(hrOut, "<!--", "-->") && containsAll(grOut, "<!--", "-->") {
		t.Logf("note: gotpl output preserves HTML comments while hbs output " +
			"loses them — opposite of what one might expect.")
	}
	if containsAll(hrOut, "<!--") && !containsAll(grOut, "<!--") {
		t.Logf("note: html/template stripped HTML comments inside <head>. " +
			"This is documented; pass content as template.HTML to preserve.")
	}
}

func containsAll(haystack string, needles ...string) bool {
	for _, n := range needles {
		if !strings.Contains(haystack, n) {
			return false
		}
	}
	return true
}

func unifiedDiff(nameA, contentA, nameB, contentB string) string {
	aLines := strings.Split(contentA, "\n")
	bLines := strings.Split(contentB, "\n")
	var out strings.Builder
	i, j := 0, 0
	for i < len(aLines) || j < len(bLines) {
		switch {
		case i >= len(aLines):
			fmt.Fprintf(&out, "> %s\n", bLines[j])
			j++
		case j >= len(bLines):
			fmt.Fprintf(&out, "< %s\n", aLines[i])
			i++
		case aLines[i] == bLines[j]:
			fmt.Fprintf(&out, "  %s\n", aLines[i])
			i++
			j++
		default:
			fmt.Fprintf(&out, "< %s\n", aLines[i])
			fmt.Fprintf(&out, "> %s\n", bLines[j])
			i++
			j++
		}
	}
	return out.String()
}

func numberedLines(s string, n int) string {
	lines := strings.Split(s, "\n")
	if len(lines) > n {
		lines = lines[:n]
	}
	var out strings.Builder
	for i, l := range lines {
		fmt.Fprintf(&out, "%3d| %s\n", i+1, l)
	}
	return out.String()
}

func trunc(s string, lines int) string {
	parts := strings.Split(s, "\n")
	if len(parts) <= lines {
		return s
	}
	return strings.Join(parts[:lines], "\n") + fmt.Sprintf("\n... (%d more lines)", len(parts)-lines)
}