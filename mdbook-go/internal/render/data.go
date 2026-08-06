package render

import (
	"html/template"
	"path/filepath"
	"strings"

	"mdbook-go/internal/book"
	"mdbook-go/internal/config"
	"mdbook-go/internal/theme"
	"mdbook-go/internal/tplgotpl"
	"mdbook-go/internal/utils"
)

// RenderData is the typed template data the production .gohtml files consume.
// It embeds tplgotpl.Env so the {{.Resource}}, {{.TocHTML}} and {{.FA}}
// helpers are available as promoted methods.
//
// Optional fields (FaviconSVG, LiveReloadEndpoint, etc.) are left at their
// zero value when the corresponding legacy data-map key is absent; the
// .gohtml files gate them with `{{if .X}}` so an empty value renders as
// "feature off".
type RenderData struct {
	*tplgotpl.Env

	// Identity / chrome.
	Language     string
	Title        string
	ChapterTitle string
	BookTitle    string
	Description  string
	DefaultTheme string
	PreferredDarkTheme string
	TextDirection string
	PathToRoot    string

	// Page flags.
	IsPrint   bool
	IsIndex   bool
	IsTocHTML bool
	BaseURL   string

	// Optional features.
	SearchEnabled          bool
	SearchJS               bool
	MathJaxSupport         bool
	PlaygroundJS           bool
	PlaygroundLineNumbers  bool
	PlaygroundCopyable     bool
	PrintEnable            bool
	FoldEnable             bool
	FoldLevel              int

	// Resources.
	AdditionalCSS []string
	AdditionalJS  []string

	// Head metadata.
	FaviconSVG string
	FaviconPNG string

	// Live reload (template.URL so html/template treats it as URL-safe).
	LiveReloadEndpoint template.URL

	// Fragment map (template.JS so JS string escapes correctly).
	FragmentMap template.JS

	// Chapter navigation.
	Previous *Nav
	Next     *Nav

	// Git integration.
	GitRepositoryURL       string
	GitRepositoryEditURL   string
	GitRepositoryIconClass string
	GitRepositoryIcon      string
}

// Nav is the shape of the Previous / Next links on a page.
type Nav struct {
	Title string
	Link  string
}

// makeData builds the shared template context in the legacy hbs-style
// map[string]any form. Callers that want a typed RenderData use BuildRenderData.
func makeData(ctx *Context, cfg *config.HtmlConfig, th *theme.Theme) map[string]any {
	data := map[string]any{}

	data["language"] = ctx.Config.Book.Language
	if data["language"] == "" {
		data["language"] = "en"
	}
	data["text_direction"] = ctx.Config.Book.RealizedTextDirection()
	data["book_title"] = ctx.Config.Book.Title
	data["description"] = ctx.Config.Book.Description
	if th.FaviconPNG != nil {
		data["favicon_png"] = "favicon.png"
	}
	if th.FaviconSVG != nil {
		data["favicon_svg"] = "favicon.svg"
	}
	if cfg.LiveReloadEndpoint != "" {
		data["live_reload_endpoint"] = cfg.LiveReloadEndpoint
	}
	data["default_theme"] = cfg.DefaultThemeName()
	data["preferred_dark_theme"] = cfg.PreferredDarkThemeName()
	if cfg.MathJaxSupport {
		data["mathjax_support"] = true
	}
	if len(cfg.AdditionalCSS) > 0 {
		data["additional_css"] = relativeToRoot(ctx.Root, cfg.AdditionalCSS)
	}
	if len(cfg.AdditionalJS) > 0 {
		data["additional_js"] = relativeToRoot(ctx.Root, cfg.AdditionalJS)
	}
	if cfg.Playground.Editable && cfg.Playground.CopyJS {
		data["playground_js"] = true
		if cfg.Playground.LineNumbers {
			data["playground_line_numbers"] = true
		}
	}
	if cfg.Playground.Copyable {
		data["playground_copyable"] = true
	}
	data["print_enable"] = cfg.Print.Enable
	data["fold_enable"] = cfg.Fold.Enable
	data["fold_level"] = int(cfg.Fold.Level)
	data["sidebar_header_nav"] = cfg.SidebarHeaderNav

	search := cfg.EffectiveSearch()
	data["search_enabled"] = search.Enable
	data["search_js"] = search.Enable && search.CopyJS

	if cfg.GitRepositoryURL != "" {
		data["git_repository_url"] = cfg.GitRepositoryURL
	}
	data["git_repository_icon"] = cfg.GitRepositoryIconName()
	data["git_repository_icon_class"] = cfg.GitRepositoryIconClass()

	data["chapters"] = chapterSummaries(ctx.Book)
	return data
}

// BuildRenderData converts a legacy hbs-style data map into the typed
// RenderData struct that the production .gohtml files consume. The struct
// embeds tplgotpl.Env so {{.Resource "x"}}, {{.TocHTML}} and {{.FA ...}} work
// as promoted methods.
func BuildRenderData(data map[string]any, noSectionLabel bool) RenderData {
	out := RenderData{
		Language:               asString(data, "language"),
		Title:                  asString(data, "title"),
		ChapterTitle:           asString(data, "chapter_title"),
		BookTitle:              asString(data, "book_title"),
		Description:            asString(data, "description"),
		DefaultTheme:           asString(data, "default_theme"),
		PreferredDarkTheme:     asString(data, "preferred_dark_theme"),
		TextDirection:          asString(data, "text_direction"),
		PathToRoot:             asString(data, "path_to_root"),
		BaseURL:                asString(data, "base_url"),
		FaviconSVG:             asString(data, "favicon_svg"),
		FaviconPNG:             asString(data, "favicon_png"),
		SearchEnabled:          asBool(data, "search_enabled"),
		SearchJS:               asBool(data, "search_js"),
		MathJaxSupport:         asBool(data, "mathjax_support"),
		PlaygroundJS:           asBool(data, "playground_js"),
		PlaygroundLineNumbers:  asBool(data, "playground_line_numbers"),
		PlaygroundCopyable:     asBool(data, "playground_copyable"),
		PrintEnable:            asBool(data, "print_enable"),
		FoldEnable:             asBool(data, "fold_enable"),
		FoldLevel:              asInt(data, "fold_level"),
		IsPrint:                asBool(data, "is_print"),
		IsIndex:                asBool(data, "is_index"),
		IsTocHTML:              asBool(data, "is_toc_html"),
		GitRepositoryURL:       asString(data, "git_repository_url"),
		GitRepositoryEditURL:   asString(data, "git_repository_edit_url"),
		GitRepositoryIconClass: asString(data, "git_repository_icon_class"),
		GitRepositoryIcon:      asString(data, "git_repository_icon"),
	}
	out.Env = &tplgotpl.Env{}
	out.Env.Path = asString(data, "path")
	out.Env.FoldEnable = asBool(data, "fold_enable")
	out.Env.FoldLevel = asInt(data, "fold_level")
	out.Env.IsTocHTML = asBool(data, "is_toc_html")
	out.Env.NoSectionLabel = noSectionLabel
	out.Env.SidebarHeaderNav = asBool(data, "sidebar_header_nav")
	if v, ok := data["live_reload_endpoint"]; ok {
		out.LiveReloadEndpoint = template.URL(asStringFromAny(v))
	}
	if v, ok := data["fragment_map"]; ok {
		out.FragmentMap = template.JS(asStringFromAny(v))
	}
	if v, ok := data["previous"]; ok {
		if m, ok := v.(map[string]any); ok {
			out.Previous = &Nav{
				Title: asString(m, "title"),
				Link:  asString(m, "link"),
			}
		}
	}
	if v, ok := data["next"]; ok {
		if m, ok := v.(map[string]any); ok {
			out.Next = &Nav{
				Title: asString(m, "title"),
				Link:  asString(m, "link"),
			}
		}
	}
	if v, ok := data["additional_css"]; ok {
		out.AdditionalCSS = asStringSlice(v)
	}
	if v, ok := data["additional_js"]; ok {
		out.AdditionalJS = asStringSlice(v)
	}
	if v, ok := data["resources"]; ok {
		if m, ok := v.(map[string]string); ok {
			out.Env.Resources = m
		}
	}
	if v, ok := data["chapters"]; ok {
		if s, ok := v.([]any); ok {
			out.Env.Chapters = s
		}
	}
	return out
}

// asString / asBool / asInt / asStringSlice / asStringFromAny are tiny
// coercion helpers that tolerate the loose data maps the hbs engine accepted.
func asString(data map[string]any, key string) string {
	if data == nil {
		return ""
	}
	v, ok := data[key]
	if !ok {
		return ""
	}
	return asStringFromAny(v)
}

func asStringFromAny(v any) string {
	s, ok := v.(string)
	if !ok {
		return ""
	}
	return s
}

func asBool(data map[string]any, key string) bool {
	if data == nil {
		return false
	}
	v, ok := data[key]
	if !ok {
		return false
	}
	b, ok := v.(bool)
	if !ok {
		return false
	}
	return b
}

func asInt(data map[string]any, key string) int {
	if data == nil {
		return 0
	}
	v, ok := data[key]
	if !ok {
		return 0
	}
	switch n := v.(type) {
	case int:
		return n
	case int64:
		return int(n)
	case float64:
		return int(n)
	case string:
		if n == "" {
			return 0
		}
		var out int
		for _, c := range n {
			if c < '0' || c > '9' {
				return 0
			}
			out = out*10 + int(c-'0')
		}
		return out
	}
	return 0
}

func asStringSlice(v any) []string {
	switch s := v.(type) {
	case []string:
		return s
	case []any:
		out := make([]string, 0, len(s))
		for _, e := range s {
			if str, ok := e.(string); ok {
				out = append(out, str)
			}
		}
		return out
	}
	return nil
}

// chapterSummaries flattens the book into the list the sidebar helper walks.
func chapterSummaries(b *book.Book) []any {
	var out []any
	var visit func(items []book.BookItem)
	visit = func(items []book.BookItem) {
		for _, item := range items {
			switch {
			case item.PartTitle != nil:
				out = append(out, map[string]any{"part": item.PartTitle.Name})
			case item.Separator != nil:
				out = append(out, map[string]any{"spacer": "_spacer_"})
			case item.Chapter != nil:
				ch := item.Chapter
				entry := map[string]any{
					"name":          ch.Name,
					"has_sub_items": boolString(len(ch.SubChapters()) > 0),
				}
				if section := ch.Number.String(); section != "" {
					entry["section"] = section
				}
				if ch.Path != "" {
					entry["path"] = ch.Path
				}
				out = append(out, entry)
				visit(ch.SubItems)
			}
		}
	}
	visit(b.Items)
	return out
}

// boolString renders a bool the way the Rust data map does: as a string, since
// the sidebar helper parses it back out of a string map.
func boolString(v bool) string {
	if v {
		return "true"
	}
	return "false"
}

// relativeToRoot strips the book root prefix from user-specified asset paths.
func relativeToRoot(root string, paths []string) []any {
	out := make([]any, 0, len(paths))
	for _, p := range paths {
		if rel, err := filepath.Rel(root, p); err == nil && !strings.HasPrefix(rel, "..") {
			out = append(out, utils.ToURLPath(rel))
			continue
		}
		out = append(out, utils.ToURLPath(p))
	}
	return out
}
