// Package render drives the HTML backend: it turns a loaded book into the
// output directory. It is a port of
// crates/mdbook-html/src/html_handlebars/hbs_renderer.rs.
package render

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"mdbook-go/internal/book"
	"mdbook-go/internal/config"
	"mdbook-go/internal/fontawesome"
	"mdbook-go/internal/hbs"
	"mdbook-go/internal/html"
	"mdbook-go/internal/search"
	"mdbook-go/internal/static"
	"mdbook-go/internal/theme"
	"mdbook-go/internal/utils"
)

// Context carries everything the HTML backend needs for one build.
type Context struct {
	// Root is the book root directory.
	Root string
	// Destination is the output directory.
	Destination string
	// Config is the full book configuration.
	Config *config.Config
	// Book is the post-preprocessing chapter tree. When no preprocessors
	// are configured it is identical to the loaded book.
	Book *book.Book
	// ChapterTitles records per-chapter title overrides set by
	// preprocessors such as `links`. Nil when no preprocessor ran.
	ChapterTitles map[string]string
}

// chapterTree pairs a chapter with its rendered node tree, which is needed more
// than once (chapter page, print page, search index).
type chapterTree struct {
	chapter *book.Chapter
	tree    *html.Node
}

// Render writes the whole book. The order of operations follows the Rust
// renderer so that generated file names, which depend on content hashes, match.
func Render(ctx *Context) error {
	htmlCfg, err := ctx.Config.HTML()
	if err != nil {
		return err
	}
	srcDir := ctx.Config.Book.SourceDir
	buildDir := filepath.Join(ctx.Root, ctx.Config.Build.BuildDir)

	if _, err := os.Stat(ctx.Destination); err == nil {
		if err := utils.RemoveDirContent(ctx.Destination); err != nil {
			return err
		}
	}
	if err := os.MkdirAll(ctx.Destination, 0o755); err != nil {
		return err
	}

	th := theme.New(htmlCfg.ThemeDir(ctx.Root))
	registry, err := newRegistry(th, htmlCfg)
	if err != nil {
		return err
	}

	data := makeData(ctx, htmlCfg, th)
	trees, err := buildTrees(ctx.Book, htmlCfg, ctx.Config.Rust.Edition)
	if err != nil {
		return err
	}

	files, err := static.New(th, htmlCfg, ctx.Root)
	if err != nil {
		return err
	}

	searchCfg := htmlCfg.EffectiveSearch()
	if searchCfg.Enable {
		if err := addSearchFiles(files, searchCfg, trees); err != nil {
			return err
		}
	}

	// toc.js is a generated asset, so it has to exist before hashing.
	tocJS, err := registry.Render("toc_js", data)
	if err != nil {
		return fmt.Errorf("render toc.js: %w", err)
	}
	files.AddBuiltin("toc.js", []byte(tocJS))

	if err := files.Hash(); err != nil {
		return err
	}
	resources, err := files.Write(ctx.Destination)
	if err != nil {
		return err
	}
	registry.RegisterHelper("resource", resourceHelper(resources))

	// toc.html is the no-JavaScript fallback sidebar.
	data["is_toc_html"] = true
	data["path"] = "toc.html"
	tocHTML, err := registry.Render("toc_html", data)
	if err != nil {
		return fmt.Errorf("render toc.html: %w", err)
	}
	if err := utils.WriteFile(filepath.Join(ctx.Destination, "toc.html"), []byte(tocHTML)); err != nil {
		return err
	}
	delete(data, "is_toc_html")
	delete(data, "path")

	if err := utils.WriteFile(filepath.Join(ctx.Destination, ".nojekyll"),
		[]byte("This file makes sure that Github Pages doesn't process mdBook's output.\n")); err != nil {
		return err
	}
	if htmlCfg.CName != "" {
		if err := utils.WriteFile(filepath.Join(ctx.Destination, "CNAME"),
			[]byte(htmlCfg.CName+"\n")); err != nil {
			return err
		}
	}

	for i, item := range trees {
		var previous, next *book.Chapter
		if i > 0 {
			previous = trees[i-1].chapter
		}
		if i < len(trees)-1 {
			next = trees[i+1].chapter
		}
		if err := renderChapter(ctx, htmlCfg, registry, data, item, previous, next, i == 0); err != nil {
			return err
		}
	}

	if htmlCfg.Render404() {
		if err := render404(ctx, htmlCfg, registry, data, srcDir); err != nil {
			return err
		}
	}

	if htmlCfg.Print.Enable {
		content := renderPrintContent(trees)
		printData := cloneData(data)
		if ctx.Config.Book.Title != "" {
			printData["title"] = ctx.Config.Book.Title
		} else {
			delete(printData, "title")
		}
		printData["is_print"] = true
		printData["path"] = "print.md"
		printData["content"] = content
		printData["path_to_root"] = utils.PathToRoot("print.md")
		page, err := registry.Render("index", printData)
		if err != nil {
			return fmt.Errorf("render print.html: %w", err)
		}
		if err := utils.WriteFile(filepath.Join(ctx.Destination, "print.html"), []byte(page)); err != nil {
			return err
		}
	}

	if err := emitRedirects(ctx.Destination, registry, htmlCfg.Redirect); err != nil {
		return fmt.Errorf("unable to emit redirects: %w", err)
	}

	// Everything in the source tree that is not Markdown is copied verbatim.
	return utils.CopyFilesExceptExt(srcDir, ctx.Destination, true, buildDir, []string{"md"})
}

// newRegistry registers the theme's templates, partials and helpers. The
// `resource` helper is added later, once the static file names are known.
func newRegistry(th *theme.Theme, cfg *config.HtmlConfig) (*hbs.Registry, error) {
	r := hbs.New()
	templates := []struct {
		name string
		src  []byte
	}{
		{"index", th.Index},
		{"redirect", th.Redirect},
		{"toc_js", th.TocJS},
		{"toc_html", th.TocHTML},
	}
	for _, t := range templates {
		if err := r.RegisterTemplate(t.name, string(t.src)); err != nil {
			return nil, fmt.Errorf("template %s: %w", t.name, err)
		}
	}
	if err := r.RegisterPartial("head", string(th.Head)); err != nil {
		return nil, err
	}
	if err := r.RegisterPartial("header", string(th.Header)); err != nil {
		return nil, err
	}
	r.RegisterBlockHelper("toc", tocHelper(cfg.NoSectionLabel))
	r.RegisterHelper("fa", faHelper)
	return r, nil
}

// faHelper implements `{{fa TYPE NAME [id]}}`.
func faHelper(_ *hbs.Context, params []any) (string, error) {
	if len(params) < 2 {
		return "", fmt.Errorf("fa helper expects at least two parameters")
	}
	iconType, err := fontawesome.TypeFromString(fmt.Sprint(params[0]))
	if err != nil {
		return "", err
	}
	id := ""
	if len(params) > 2 {
		id = fmt.Sprint(params[2])
	}
	return fontawesome.Span(iconType, fmt.Sprint(params[1]), id)
}

// resourceHelper implements `{{ resource "name" }}`: the emitted asset name,
// prefixed with enough `../` to reach the output root from the current page.
func resourceHelper(resources map[string]string) hbs.Helper {
	return func(ctx *hbs.Context, params []any) (string, error) {
		if len(params) < 1 {
			return "", fmt.Errorf("resource helper expects a name")
		}
		name := fmt.Sprint(params[0])
		basePath := ""
		if v, ok := ctx.Lookup("@root/path"); ok {
			basePath = strings.ReplaceAll(fmt.Sprint(v), `"`, "")
		}
		resolved, ok := resources[name]
		if !ok {
			resolved = name
		}
		return utils.PathToRoot(basePath) + resolved, nil
	}
}

// buildTrees renders every non-draft chapter's Markdown into a node tree.
func buildTrees(b *book.Book, cfg *config.HtmlConfig, edition string) ([]*chapterTree, error) {
	var trees []*chapterTree
	for _, ch := range b.Chapters() {
		if ch.IsDraft() {
			continue
		}
		opts := html.Options{
			Path:             ch.Path,
			SmartPunctuation: cfg.SmartPunctuation,
			DefinitionLists:  cfg.DefinitionLists,
			Admonitions:      cfg.Admonitions,
			MathJax:          cfg.MathJaxSupport,
			Playground: html.PlaygroundOptions{
				Editable:    cfg.Playground.Editable,
				CopyJS:      cfg.Playground.CopyJS,
				Copyable:    cfg.Playground.Copyable,
				LineNumbers: cfg.Playground.LineNumbers,
				Runnable:    cfg.Playground.Runnable,
			},
			HideLines: cfg.Code.HideLines,
			Edition:   edition,
		}
		tree, err := html.BuildTree(ch.Content, opts)
		if err != nil {
			return nil, fmt.Errorf("render %s: %w", ch.Path, err)
		}
		trees = append(trees, &chapterTree{chapter: ch, tree: tree})
	}
	return trees, nil
}

// renderChapter writes one chapter page, and index.html for the first chapter.
func renderChapter(ctx *Context, cfg *config.HtmlConfig, registry *hbs.Registry,
	base map[string]any, item *chapterTree, previous, next *book.Chapter, isFirst bool) error {

	ch := item.chapter
	data := cloneData(base)

	if cfg.EditURLTemplate != "" {
		// {path} is replaced with the chapter path relative to the book root,
		// i.e. the configured `src` directory plus the chapter's source path.
		srcRel, err := filepath.Rel(ctx.Root, ctx.Config.Book.SourceDir)
		if err != nil {
			srcRel = ctx.Config.Book.SourceDir
		}
		source := filepath.ToSlash(filepath.Join(srcRel, ch.SourcePath))
		data["git_repository_edit_url"] = strings.ReplaceAll(cfg.EditURLTemplate, "{path}", source)
	}

	displayName := ch.Name
	if override, ok := ctx.ChapterTitles[ch.Path]; ok && override != "" {
		displayName = override
	}
	title := displayName
	if ctx.Config.Book.Title != "" {
		title = displayName + " - " + ctx.Config.Book.Title
	}
	data["path"] = ch.Path
	data["content"] = html.Serialize(item.tree)
	data["chapter_title"] = displayName
	data["title"] = title
	data["path_to_root"] = utils.PathToRoot(ch.Path)
	if section := ch.Number.String(); section != "" {
		data["section"] = section
	}
	if previous != nil {
		data["previous"] = map[string]any{
			"title": previous.Name,
			"link":  utils.ToURLPath(previous.HTMLPath()),
		}
	}
	if next != nil {
		data["next"] = map[string]any{
			"title": next.Name,
			"link":  utils.ToURLPath(next.HTMLPath()),
		}
	}

	page, err := registry.Render("index", data)
	if err != nil {
		return fmt.Errorf("render %s: %w", ch.Path, err)
	}
	if err := utils.WriteFile(filepath.Join(ctx.Destination, filepath.FromSlash(ch.HTMLPath())), []byte(page)); err != nil {
		return err
	}

	if isFirst {
		// index.html is the first chapter re-rendered as if it lived at the
		// book root.
		data["path"] = "index.md"
		data["path_to_root"] = ""
		data["is_index"] = true
		page, err := registry.Render("index", data)
		if err != nil {
			return fmt.Errorf("render index.html: %w", err)
		}
		if err := utils.WriteFile(filepath.Join(ctx.Destination, "index.html"), []byte(page)); err != nil {
			return err
		}
	}
	return nil
}

// default404 is used when the book has no 404 source file.
const default404 = "# Document not found (404)\n\nThis URL is invalid, sorry. " +
	"Please use the navigation bar or search to continue."

func render404(ctx *Context, cfg *config.HtmlConfig, registry *hbs.Registry,
	base map[string]any, srcDir string) error {

	source := default404
	name := "404.md"
	if cfg.Input404 != nil {
		name = *cfg.Input404
	}
	if data, err := os.ReadFile(filepath.Join(srcDir, filepath.FromSlash(name))); err == nil {
		source = string(data)
	}

	content, err := html.Render(source, html.Options{
		Path:             "404.md",
		SmartPunctuation: cfg.SmartPunctuation,
		DefinitionLists:  cfg.DefinitionLists,
		Admonitions:      cfg.Admonitions,
		Edition:          ctx.Config.Rust.Edition,
	})
	if err != nil {
		return err
	}

	data := cloneData(base)
	baseURL := "/"
	if cfg.SiteURL != "" {
		baseURL = cfg.SiteURL
	}
	data["base_url"] = baseURL
	data["path"] = "404.md"
	data["content"] = content
	if ctx.Config.Book.Title != "" {
		data["title"] = "Page not found - " + ctx.Config.Book.Title
	} else {
		data["title"] = "Page not found"
	}

	page, err := registry.Render("index", data)
	if err != nil {
		return fmt.Errorf("render 404: %w", err)
	}
	return utils.WriteFile(filepath.Join(ctx.Destination, cfg.Get404OutputFile()), []byte(page))
}

// addSearchFiles builds the elasticlunr index and registers the search assets.
func addSearchFiles(files *static.Files, cfg config.Search, trees []*chapterTree) error {
	docs := collectSearchDocs(cfg, trees)
	opts := search.Options{
		LimitResults:    cfg.LimitResults,
		TeaserWordCount: cfg.TeaserWordCount,
		BoostTitle:      cfg.BoostTitle,
		BoostParagraph:  cfg.BoostParagraph,
		BoostHierarchy:  cfg.BoostHierarchy,
		UseBooleanAnd:   cfg.UseBooleanAnd,
		Expand:          cfg.Expand,
	}
	if cfg.CopyJS {
		index, err := search.JS(docs, opts)
		if err != nil {
			return err
		}
		files.AddBuiltin("searchindex.js", []byte(index))
		files.AddBuiltin("searcher.js", theme.SearcherJS)
		files.AddBuiltin("mark.min.js", theme.MarkJS)
		files.AddBuiltin("elasticlunr.min.js", theme.ElasticlunrJS)
	}
	return nil
}

// cloneData copies the shared template data so per-page mutations do not leak
// into later pages.
func cloneData(data map[string]any) map[string]any {
	out := make(map[string]any, len(data)+8)
	for k, v := range data {
		out[k] = v
	}
	return out
}
