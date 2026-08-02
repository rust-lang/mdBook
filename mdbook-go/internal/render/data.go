package render

import (
	"path/filepath"
	"strings"

	"mdbook-go/internal/book"
	"mdbook-go/internal/config"
	"mdbook-go/internal/theme"
	"mdbook-go/internal/utils"
)

// makeData builds the template context shared by every page. It is a port of
// make_data in crates/mdbook-html/src/html_handlebars/hbs_renderer.rs; keys are
// only inserted when the Rust code inserts them, because the templates test for
// their presence.
func makeData(ctx *Context, cfg *config.HtmlConfig, th *theme.Theme) map[string]any {
	data := map[string]any{}

	data["language"] = ctx.Config.Book.Language
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
	// The icon keys are always present in the Rust data map, even without a
	// repository URL; the template only reads them inside the URL check.
	data["git_repository_icon"] = cfg.GitRepositoryIconName()
	data["git_repository_icon_class"] = cfg.GitRepositoryIconClass()

	data["chapters"] = chapterSummaries(ctx.Book)
	return data
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
