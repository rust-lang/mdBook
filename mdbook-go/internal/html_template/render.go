// Package html_template is the HTML backend renderer: it turns a loaded
// book into the output directory. It is the aggregate Go port of
// crates/mdbook-html/src/html_handlebars/ (hbs_renderer.rs, search.rs,
// static_files.rs, helpers/) plus the theme asset resolver
// (crates/mdbook-html/src/theme/mod.rs).
//
// The Handlebars engine those files were written against is replaced by the
// standard library html/template: engine.go holds the Registry wrapper,
// helpers.go the Env passed into templates, and templates/ the five .html
// files that replaced theme/templates/*.hbs (the legacy internal/hbs engine
// was removed on 2026-08-08).
package html_template

import (
	"fmt"
	"html/template"
	"os"
	"path/filepath"
	"strings"

	"mdbook-go/internal/html"
	"mdbook-go/internal/model"
	"mdbook-go/pkg/fs"
)

// Context carries everything the HTML backend needs for one build.
type Context struct {
	Root          string
	Destination   string
	Config        *model.Config
	Book          *model.Book
	ChapterTitles map[string]string
}

type chapterTree struct {
	chapter *model.Chapter
	tree    *html.Node
}

// Render writes the whole book. The order of operations follows the Rust
// renderer so that generated file names, which depend on content hashes, match.
func Render(ctx *Context) error {
	htmlCfg, err := ctx.Config.HTML()
	if err != nil {
		return err
	}
	srcDir := ctx.Config.Package.Root
	buildDir := filepath.Join(ctx.Root, ctx.Config.Build.BuildDir)

	if _, err := os.Stat(ctx.Destination); err == nil {
		if err := fs.RemoveDirContent(ctx.Destination); err != nil {
			return err
		}
	}
	if err := os.MkdirAll(ctx.Destination, 0o755); err != nil {
		return err
	}

	th := NewTheme(htmlCfg.ThemeDir(ctx.Root))
	registry, err := newRegistry(th, htmlCfg)
	if err != nil {
		return err
	}

	data := makeData(ctx, htmlCfg, th)
	trees, err := buildTrees(ctx.Book, htmlCfg)
	if err != nil {
		return err
	}

	files, err := NewStaticFiles(th, htmlCfg, ctx.Root)
	if err != nil {
		return err
	}

	searchCfg := htmlCfg.EffectiveSearch()
	if searchCfg.Enable {
		if err := addSearchFiles(files, searchCfg, trees); err != nil {
			return err
		}
	}

	// toc.js is a generated asset, so it has to exist before hashing. The
	// source is built directly in Go because the body contains the sidebar
	// injection inside a single-quoted JS string literal, which html/template
	// cannot evaluate. The sidebar HTML and the optional header-tracking IIFE
	// are spliced in by string concatenation.
	tocJS, err := buildTocJS(data, htmlCfg, ctx.Book)
	if err != nil {
		return fmt.Errorf("build toc.js: %w", err)
	}
	files.AddBuiltin("toc.js", []byte(tocJS))

	if err := files.Hash(); err != nil {
		return err
	}
	resources, err := files.Write(ctx.Destination)
	if err != nil {
		return err
	}
	data["resources"] = resources
	// Rebuild the typed view now that Resources is populated.
	view := BuildRenderData(data, false)

	// toc.html is the no-JavaScript fallback sidebar.
	tocView := view
	tocView.IsTocHTML = true
	tocView.Path = "toc.html"
	tocView.PathToRoot = fs.PathToRoot("toc.html")
	tocHTML, err := registry.Render("toc.html", tocView)
	if err != nil {
		return fmt.Errorf("render toc.html: %w", err)
	}
	if err := fs.WriteFile(filepath.Join(ctx.Destination, "toc.html"), []byte(tocHTML)); err != nil {
		return err
	}

	if err := fs.WriteFile(filepath.Join(ctx.Destination, ".nojekyll"),
		[]byte("This file makes sure that Github Pages doesn't process mdBook's output.\n")); err != nil {
		return err
	}
	if htmlCfg.CName != "" {
		if err := fs.WriteFile(filepath.Join(ctx.Destination, "CNAME"),
			[]byte(htmlCfg.CName+"\n")); err != nil {
			return err
		}
	}

	for i, item := range trees {
		var previous, next *model.Chapter
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

	if err := emitRedirects(ctx.Destination, registry, htmlCfg.Redirect); err != nil {
		return fmt.Errorf("unable to emit redirects: %w", err)
	}

	return fs.CopyFilesExceptExt(srcDir, ctx.Destination, true, buildDir, []string{"md"})
}

func newRegistry(th *Theme, cfg *model.HtmlConfig) (*Registry, error) {
	r := NewRegistry()
	_ = th
	_ = cfg

	if err := r.RegisterPartial("head", `{{/* Put your head HTML text here */}}`); err != nil {
		return nil, err
	}
	if err := r.RegisterPartial("header", `{{/* Put your header HTML text here */}}`); err != nil {
		return nil, err
	}
	if err := r.LoadProduction(); err != nil {
		return nil, err
	}
	return r, nil
}

// buildTocJS assembles the toc.js source. The sidebar HTML is computed by
// tplgotpl.Env.TocHTML() (the same path that produced the {{#toc}} block
// under the legacy engine); the optional header-tracking IIFE is spliced
// in when SidebarHeaderNav is enabled.
//
// tocJSPrefix / tocJSMiddle / tocJSAfterClass are the three slices of
// toc.js that surround the variable parts (sidebar HTML, path_to_root,
// optional IIFE). Keeping them as constants keeps buildTocJS short and
// makes the differences with the source obvious.
func buildTocJS(data map[string]any, cfg *model.HtmlConfig, b *model.Book) (string, error) {
	chapters, _ := data["chapters"].([]any)
	foldEnable, _ := data["fold_enable"].(bool)
	foldLevel := asInt(data, "fold_level")
	noSectionLabel := cfg.NoSectionLabel
	if _, ok := data["no_section_label"]; ok {
		noSectionLabel = asBool(data, "no_section_label")
	}
	env := Env{
		Chapters:       chapters,
		FoldEnable:     foldEnable,
		FoldLevel:      foldLevel,
		NoSectionLabel: noSectionLabel,
	}
	sidebarHTML := string(env.TocHTML())

	var out strings.Builder
	out.WriteString(tocJSPrefix)
	out.WriteString("\n        this.innerHTML = '")
	out.WriteString(escapeForJSSingleQuoted(sidebarHTML))
	out.WriteString("';\n")
	out.WriteString(tocJSMiddle)
	out.WriteString(tocJSAfterClass)

	if cfg.SidebarHeaderNav {
		out.WriteString("\n")
		out.WriteString(SidebarHeaderNavSource)
	}
	out.WriteString("\n")
	return out.String(), nil
}

const tocJSPrefix = `// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {`

const tocJSMiddle = `        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split('#')[0].split('?')[0];
        if (current_page.endsWith('/')) {
            current_page += 'index.html';
        }
        const links = Array.prototype.slice.call(this.querySelectorAll('a'));
        const l = links.length;
        for (let i = 0; i < l; ++i) {
            const link = links[i];
            const href = link.getAttribute('href');
            if (href && !href.startsWith('#') && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;`

const tocJSAfterClass = `
            }
            // The 'index' page is supposed to alias the first chapter in the book.
            // Check both with and without the '.html' suffix to be robust against pretty URLs
            if (link.href.replace(/\.html$/, '') === current_page.replace(/\.html$/, '')
                || i === 0
                && path_to_root === ''
                && current_page.endsWith('/index.html')) {
                link.classList.add('active');
                let parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', e => {
            if (e.target.tagName === 'A') {
                const clientRect = e.target.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                sessionStorage.setItem('sidebar-scroll-offset', clientRect.top - sidebarRect.top);
            }
        }, { passive: true });
        const sidebarScrollOffset = sessionStorage.getItem('sidebar-scroll-offset');
        sessionStorage.removeItem('sidebar-scroll-offset');
        if (sidebarScrollOffset !== null) {
            // preserve sidebar scroll position when navigating via links within sidebar
            const activeSection = this.querySelector('.active');
            if (activeSection) {
                const clientRect = activeSection.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                const currentOffset = clientRect.top - sidebarRect.top;
                this.scrollTop += currentOffset - parseFloat(sidebarScrollOffset);
            }
        } else {
            // scroll sidebar to current active section when navigating via
            // 'next/previous chapter' buttons
            const activeSection = document.querySelector('#mdbook-sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        const sidebarAnchorToggles = document.querySelectorAll('.chapter-fold-toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(el => {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define('mdbook-sidebar-scrollbox', MDBookSidebarScrollbox);`

// escapeForJSSingleQuoted escapes s for use inside a JS single-quoted string
// literal. We need to escape `\` and `'` and replace newlines with `\n` (so
// the rendered toc.js remains a single-line script for the embed step).
func escapeForJSSingleQuoted(s string) string {
	r := strings.NewReplacer(
		`\`, `\\`,
		`'`, `\'`,
		"\n", `\n`,
		"\r", `\r`,
	)
	return r.Replace(s)
}

// buildTrees renders every non-draft chapter's Markdown into a node tree.
func buildTrees(b *model.Book, cfg *model.HtmlConfig) ([]*chapterTree, error) {
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
			HideLines:        cfg.Code.HideLines,
		}
		tree, err := html.BuildTree(ch.Content, opts)
		if err != nil {
			return nil, fmt.Errorf("render %s: %w", ch.Path, err)
		}
		trees = append(trees, &chapterTree{chapter: ch, tree: tree})
	}
	return trees, nil
}

func renderChapter(ctx *Context, cfg *model.HtmlConfig, registry *Registry,
	base map[string]any, item *chapterTree, previous, next *model.Chapter, isFirst bool) error {

	ch := item.chapter
	data := cloneData(base)

	if cfg.EditURLTemplate != "" {
		srcRel, err := filepath.Rel(ctx.Root, ctx.Config.Package.Root)
		if err != nil {
			srcRel = ctx.Config.Package.Root
		}
		source := filepath.ToSlash(filepath.Join(srcRel, ch.SourcePath))
		data["git_repository_edit_url"] = strings.ReplaceAll(cfg.EditURLTemplate, "{path}", source)
	}

	displayName := ch.Name
	if override, ok := ctx.ChapterTitles[ch.Path]; ok && override != "" {
		displayName = override
	}
	title := displayName
	if ctx.Config.Package.Title != "" {
		title = displayName + " - " + ctx.Config.Package.Title
	}
	data["path"] = ch.Path
	data["content"] = template.HTML(html.Serialize(item.tree))
	data["chapter_title"] = displayName
	data["title"] = title
	data["path_to_root"] = fs.PathToRoot(ch.Path)
	if section := ch.Number.String(); section != "" {
		data["section"] = section
	}
	if previous != nil {
		data["previous"] = map[string]any{
			"title": previous.Name,
			"link":  fs.ToURLPath(previous.HTMLPath()),
		}
	}
	if next != nil {
		data["next"] = map[string]any{
			"title": next.Name,
			"link":  fs.ToURLPath(next.HTMLPath()),
		}
	}

	page, err := registry.Render("index", BuildRenderData(data, false))
	if err != nil {
		return fmt.Errorf("render %s: %w", ch.Path, err)
	}
	if err := fs.WriteFile(filepath.Join(ctx.Destination, filepath.FromSlash(ch.HTMLPath())), []byte(page)); err != nil {
		return err
	}

	if isFirst {
		data["path"] = "index.md"
		data["path_to_root"] = ""
		data["is_index"] = true
		page, err := registry.Render("index", BuildRenderData(data, false))
		if err != nil {
			return fmt.Errorf("render index.html: %w", err)
		}
		if err := fs.WriteFile(filepath.Join(ctx.Destination, "index.html"), []byte(page)); err != nil {
			return err
		}
	}
	return nil
}

const default404 = "# Document not found (404)\n\nThis URL is invalid, sorry. " +
	"Please use the navigation bar or search to continue."

func render404(ctx *Context, cfg *model.HtmlConfig, registry *Registry,
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
	data["content"] = template.HTML(content)
	if ctx.Config.Package.Title != "" {
		data["title"] = "Page not found - " + ctx.Config.Package.Title
	} else {
		data["title"] = "Page not found"
	}

	page, err := registry.Render("index", BuildRenderData(data, false))
	if err != nil {
		return fmt.Errorf("render 404: %w", err)
	}
	return fs.WriteFile(filepath.Join(ctx.Destination, cfg.Get404OutputFile()), []byte(page))
}

func addSearchFiles(files *Files, cfg model.Search, trees []*chapterTree) error {
	docs := collectSearchDocs(cfg, trees)
	opts := Options{
		LimitResults:    cfg.LimitResults,
		TeaserWordCount: cfg.TeaserWordCount,
		BoostTitle:      cfg.BoostTitle,
		BoostParagraph:  cfg.BoostParagraph,
		BoostHierarchy:  cfg.BoostHierarchy,
		UseBooleanAnd:   cfg.UseBooleanAnd,
		Expand:          cfg.Expand,
	}
	if cfg.CopyJS {
		index, err := JS(docs, opts)
		if err != nil {
			return err
		}
		files.AddBuiltin("searchindex.js", []byte(index))
		files.AddBuiltin("searcher.js", SearcherJS)
		files.AddBuiltin("mark.min.js", MarkJS)
		files.AddBuiltin("elasticlunr.min.js", ElasticlunrJS)
	}
	return nil
}

func cloneData(data map[string]any) map[string]any {
	out := make(map[string]any, len(data)+8)
	for k, v := range data {
		out[k] = v
	}
	return out
}
