// Package theme resolves the asset set used to render a book: the embedded
// defaults, overridden file-by-file from the user's theme directory. It is a
// port of crates/mdbook-html/src/theme/mod.rs.
package theme

import (
	"os"
	"path/filepath"

	themedata "mdbook-go/theme"
)

// Theme carries the resolved contents of every themeable asset.
type Theme struct {
	Index    []byte
	Head     []byte
	Redirect []byte
	Header   []byte
	TocJS    []byte
	TocHTML  []byte

	ChromeCSS    []byte
	GeneralCSS   []byte
	VariablesCSS []byte

	// FontsCSS is nil unless the user supplied theme/fonts/fonts.css. When it
	// is nil the renderer emits the bundled font set instead.
	FontsCSS []byte
	// FontFiles holds absolute paths to extra files in the user's theme/fonts/.
	FontFiles []string

	// FaviconPNG and FaviconSVG are nil when the asset should not be emitted.
	FaviconPNG []byte
	FaviconSVG []byte

	JS               []byte
	HighlightCSS     []byte
	TomorrowNightCSS []byte
	AyuHighlightCSS  []byte
	HighlightJS      []byte
	ClipboardJS      []byte

	// GitHubMarkdownCSS variants back the chapter body styling via .markdown-body
	// (a separately generated stylesheet that replaces the legacy chrome CSS for
	// the article region). Emitted unconditionally so the toggle works.
	GitHubMarkdownLightCSS []byte
	GitHubMarkdownDarkCSS  []byte
}

// Bundled font assets, in the same order as theme/fonts.rs.
var (
	// FontsCSSDefault is the bundled fonts.css template.
	FontsCSSDefault = themedata.MustRead("fonts/fonts.css")
	// FontLicenses lists the bundled license files, which are never hashed.
	FontLicenses = []NamedAsset{
		{"fonts/OPEN-SANS-LICENSE.txt", themedata.MustRead("fonts/OPEN-SANS-LICENSE.txt")},
		{"fonts/SOURCE-CODE-PRO-LICENSE.txt", themedata.MustRead("fonts/SOURCE-CODE-PRO-LICENSE.txt")},
	}
	// OpenSans lists the bundled Open Sans woff2 files.
	OpenSans = openSans()
	// SourceCodePro is the bundled monospace font.
	SourceCodePro = NamedAsset{
		"fonts/source-code-pro-v11-all-charsets-500.woff2",
		themedata.MustRead("fonts/source-code-pro-v11-all-charsets-500.woff2"),
	}
	// PlaygroundEditor (Ace editor bundle) was removed — the editable code-block
	// feature is not currently used. If reintroduced, restore the original list.
	// SearcherJS, MarkJS and ElasticlunrJS back the search UI.
	SearcherJS    = themedata.MustRead("searcher/searcher.js")
	MarkJS        = themedata.MustRead("searcher/mark.min.js")
	ElasticlunrJS = themedata.MustRead("searcher/elasticlunr.min.js")
)

// NamedAsset is an output filename paired with its contents.
type NamedAsset struct {
	Name string
	Data []byte
}

func openSans() []NamedAsset {
	names := []string{
		"300", "300italic", "regular", "italic", "600",
		"600italic", "700", "700italic", "800", "800italic",
	}
	out := make([]NamedAsset, 0, len(names))
	for _, n := range names {
		file := "fonts/open-sans-v17-all-charsets-" + n + ".woff2"
		out = append(out, NamedAsset{file, themedata.MustRead(file)})
	}
	return out
}

// Default returns the embedded theme with no user overrides applied.
func Default() *Theme {
	return &Theme{
		Index:            themedata.MustRead("templates/index.hbs"),
		Head:             themedata.MustRead("templates/head.hbs"),
		Redirect:         themedata.MustRead("templates/redirect.hbs"),
		Header:           themedata.MustRead("templates/header.hbs"),
		TocJS:            themedata.MustRead("templates/toc.js.hbs"),
		TocHTML:          themedata.MustRead("templates/toc.html.hbs"),
		ChromeCSS:        themedata.MustRead("css/chrome.css"),
		GeneralCSS:       themedata.MustRead("css/general.css"),
		VariablesCSS:     themedata.MustRead("css/variables.css"),
		FaviconPNG:       themedata.MustRead("images/favicon.png"),
		FaviconSVG:       themedata.MustRead("images/favicon.svg"),
		JS:               themedata.MustRead("js/book.js"),
		HighlightCSS:     themedata.MustRead("css/highlight.css"),
		TomorrowNightCSS: themedata.MustRead("css/tomorrow-night.css"),
		AyuHighlightCSS:  themedata.MustRead("css/ayu-highlight.css"),
		HighlightJS:      themedata.MustRead("js/highlight.js"),
		ClipboardJS:      themedata.MustRead("js/clipboard.min.js"),
		GitHubMarkdownLightCSS: themedata.MustRead("css/github-markdown-light.css"),
		GitHubMarkdownDarkCSS:  themedata.MustRead("css/github-markdown-dark.css"),
	}
}

// New loads the defaults and then overrides individual files from themeDir.
// A missing directory or a missing file is not an error.
func New(themeDir string) *Theme {
	t := Default()
	info, err := os.Stat(themeDir)
	if err != nil || !info.IsDir() {
		return t
	}

	overrides := []struct {
		rel  string
		dest *[]byte
	}{
		{"index.hbs", &t.Index},
		{"head.hbs", &t.Head},
		{"redirect.hbs", &t.Redirect},
		{"header.hbs", &t.Header},
		{"toc.js.hbs", &t.TocJS},
		{"toc.html.hbs", &t.TocHTML},
		{"book.js", &t.JS},
		{"css/chrome.css", &t.ChromeCSS},
		{"css/general.css", &t.GeneralCSS},
		{"css/variables.css", &t.VariablesCSS},
		{"highlight.js", &t.HighlightJS},
		{"clipboard.min.js", &t.ClipboardJS},
		{"highlight.css", &t.HighlightCSS},
		{"tomorrow-night.css", &t.TomorrowNightCSS},
		{"ayu-highlight.css", &t.AyuHighlightCSS},
	}
	for _, o := range overrides {
		loadInto(filepath.Join(themeDir, filepath.FromSlash(o.rel)), o.dest)
	}

	fontsDir := filepath.Join(themeDir, "fonts")
	if _, err := os.Stat(fontsDir); err == nil {
		var fontsCSS []byte
		if loadInto(filepath.Join(fontsDir, "fonts.css"), &fontsCSS) {
			t.FontsCSS = fontsCSS
		}
		if entries, err := os.ReadDir(fontsDir); err == nil {
			for _, entry := range entries {
				if entry.Name() == "fonts.css" || entry.IsDir() {
					continue
				}
				t.FontFiles = append(t.FontFiles, filepath.Join(fontsDir, entry.Name()))
			}
		}
	}

	// When the user overrides exactly one favicon, the other default is
	// dropped rather than emitted alongside it.
	png := loadInto(filepath.Join(themeDir, "favicon.png"), &t.FaviconPNG)
	svg := loadInto(filepath.Join(themeDir, "favicon.svg"), &t.FaviconSVG)
	switch {
	case png && !svg:
		t.FaviconSVG = nil
	case !png && svg:
		t.FaviconPNG = nil
	}

	return t
}

// loadInto replaces *dest with the file contents and reports whether it did.
func loadInto(path string, dest *[]byte) bool {
	data, err := os.ReadFile(path)
	if err != nil {
		return false
	}
	*dest = data
	return true
}

// Copy writes the default theme files into themeDir, mirroring
// Theme::copy_theme.
func Copy(themeDir string) error {
	write := func(rel string, data []byte) error {
		path := filepath.Join(themeDir, filepath.FromSlash(rel))
		if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
			return err
		}
		return os.WriteFile(path, data, 0o644)
	}
	files := []NamedAsset{
		{"book.js", themedata.MustRead("js/book.js")},
		{"favicon.png", themedata.MustRead("images/favicon.png")},
		{"favicon.svg", themedata.MustRead("images/favicon.svg")},
		{"highlight.css", themedata.MustRead("css/highlight.css")},
		{"highlight.js", themedata.MustRead("js/highlight.js")},
		{"index.hbs", themedata.MustRead("templates/index.hbs")},
		{"css/general.css", themedata.MustRead("css/general.css")},
		{"css/chrome.css", themedata.MustRead("css/chrome.css")},
		{"css/variables.css", themedata.MustRead("css/variables.css")},
	}
	files = append(files, NamedAsset{"fonts/fonts.css", FontsCSSDefault})
	files = append(files, FontLicenses...)
	files = append(files, OpenSans...)
	files = append(files, SourceCodePro)

	for _, f := range files {
		if err := write(f.Name, f.Data); err != nil {
			return err
		}
	}
	return nil
}
