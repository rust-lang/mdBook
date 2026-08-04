package render

import (
	"strconv"
	"strings"

	"mdbook-go/internal/hbs"
	"mdbook-go/internal/utils"
)

// tocHelper implements the `{{#toc}}{{/toc}}` block helper, which renders the
// sidebar. It is a port of RenderToc in
// crates/mdbook-html/src/html_handlebars/helpers/toc.rs, including its quirks:
// part titles are emitted as an `<li>` nested inside the item `<li>`, and the
// parent `<li>` is not closed after a nested `<ol class="section">`.
func tocHelper(noSectionLabel bool) hbs.BlockHelper {
	return func(ctx *hbs.Context, _ []any, _ func(map[string]any) (string, error)) (string, error) {
		chapters, _ := ctx.Lookup("@root/chapters")
		foldEnable, _ := ctx.Lookup("@root/fold_enable")
		foldLevel, _ := ctx.Lookup("@root/fold_level")
		isTocHTML, _ := ctx.Lookup("@root/is_toc_html")

		fold, _ := foldEnable.(bool)
		level := toInt(foldLevel)
		iframe, _ := isTocHTML.(bool)

		var out strings.Builder
		out.WriteString(`<ol class="chapter">`)

		currentLevel := 1
		first := true
		for _, raw := range toSlice(chapters) {
			item := toStringMap(raw)

			itemLevel := 1
			if section, ok := item["section"]; ok {
				itemLevel = strings.Count(section, ".")
			}
			expanded := !fold || itemLevel-1 < level

			switch {
			case itemLevel > currentLevel:
				// The summary parser only ever descends one level at a time.
				currentLevel++
				out.WriteString(`<ol class="section">`)
				writeLiOpenTag(&out, expanded)
			case itemLevel < currentLevel:
				for itemLevel < currentLevel {
					out.WriteString("</li></ol>")
					currentLevel--
				}
				writeLiOpenTag(&out, expanded)
			default:
				if !first {
					out.WriteString("</li>")
				}
				writeLiOpenTag(&out, expanded)
			}
			first = false

			if _, isSpacer := item["spacer"]; isSpacer {
				out.WriteString(`<li class="spacer"></li>`)
				continue
			}
			if title, isPart := item["part"]; isPart {
				out.WriteString(`<li class="part-title">`)
				out.WriteString(utils.EscapeHTMLAttribute(title))
				out.WriteString("</li>")
				continue
			}

			out.WriteString(`<span class="chapter-link-wrapper">`)

			path, hasPath := item["path"]
			if hasPath && path != "" {
				out.WriteString(`<a href="`)
				// Rust's TOC omits any leading `./` on chapter hrefs; align.
				path = strings.TrimPrefix(path, "./")
				out.WriteString(utils.ToURLPath(withHTMLExtension(path)))
				if iframe {
					out.WriteString(`" target="_parent">`)
				} else {
					out.WriteString(`">`)
				}
			} else {
				hasPath = false
				out.WriteString("<span>")
			}

			if !noSectionLabel {
				if section, ok := item["section"]; ok {
					out.WriteString(`<strong aria-hidden="true">`)
					out.WriteString(section)
					out.WriteString("</strong> ")
				}
			}
			if name, ok := item["name"]; ok {
				out.WriteString(utils.EscapeHTMLAttribute(name))
			}
			if hasPath {
				out.WriteString("</a>")
			} else {
				out.WriteString("</span>")
			}

			if flag, ok := item["has_sub_items"]; ok && fold && flag == "true" {
				out.WriteString(`<a class="chapter-fold-toggle"><div>❱</div></a>`)
			}
			out.WriteString("</span>")
		}
		for currentLevel > 0 {
			out.WriteString("</li></ol>")
			currentLevel--
		}
		return out.String(), nil
	}
}

func writeLiOpenTag(out *strings.Builder, expanded bool) {
	out.WriteString(`<li class="chapter-item `)
	if expanded {
		out.WriteString("expanded ")
	}
	out.WriteString(`">`)
}

// withHTMLExtension replaces a path's extension with `.html`, matching Rust's
// Path::with_extension.
func withHTMLExtension(path string) string {
	slash := strings.LastIndexByte(path, '/')
	dot := strings.LastIndexByte(path, '.')
	if dot > slash {
		return path[:dot] + ".html"
	}
	return path + ".html"
}

func toSlice(v any) []any {
	if s, ok := v.([]any); ok {
		return s
	}
	return nil
}

func toStringMap(v any) map[string]string {
	out := map[string]string{}
	m, ok := v.(map[string]any)
	if !ok {
		return out
	}
	for k, value := range m {
		if s, ok := value.(string); ok {
			out[k] = s
		}
	}
	return out
}

func toInt(v any) int {
	switch n := v.(type) {
	case int:
		return n
	case int64:
		return int(n)
	case uint8:
		return int(n)
	case float64:
		return int(n)
	case string:
		i, _ := strconv.Atoi(n)
		return i
	}
	return 0
}
