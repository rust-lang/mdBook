package render

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"mdbook-go/internal/hbs"
	"mdbook-go/internal/html"
	"mdbook-go/internal/utils"
)

// printSeparator is placed between chapters on the print page.
const printSeparator = `<div style="break-before: page; page-break-before: always;"></div>`

// renderPrintContent concatenates every chapter into the single-page print
// version. Identifiers are made unique across the whole book and cross-chapter
// links are rewritten to in-page anchors, as render_print_page does in
// crates/mdbook-html/src/html/print.rs.
func renderPrintContent(trees []*chapterTree) string {
	ids := utils.NewIDSet()
	// remap maps a chapter's html path to its old-id -> new-id table.
	remap := make(map[string]map[string]string, len(trees))
	// rootIDs maps a chapter's html path to the anchor its title occupies.
	rootIDs := make(map[string]string, len(trees))

	for _, item := range trees {
		path := utils.ToURLPath(item.chapter.HTMLPath())
		table := map[string]string{}
		for _, el := range item.tree.Elements(func(string) bool { return true }) {
			old, ok := el.El.Attr("id")
			if !ok {
				continue
			}
			unique := ids.Unique(old)
			if unique != old {
				el.El.SetAttr("id", unique)
			}
			table[old] = unique
		}
		remap[path] = table

		// Every chapter needs a heading to link to; synthesise one when the
		// chapter does not open with an <h1>.
		if id, ok := firstHeadingID(item.tree); ok {
			rootIDs[path] = id
		} else {
			id := ids.Unique(utils.IDFromContent(item.chapter.Name))
			heading := html.NewElement("h1")
			heading.El.SetAttr("id", id)
			anchor := html.NewElement("a")
			anchor.El.SetAttr("href", "#"+id)
			anchor.El.SetAttr("class", "header")
			anchor.Append(html.NewText(item.chapter.Name))
			heading.Append(anchor)
			item.tree.Prepend(heading)
			rootIDs[path] = id
		}
	}

	for _, item := range trees {
		rewritePrintLinks(item.tree, utils.ToURLPath(item.chapter.HTMLPath()), remap, rootIDs)
	}

	// Chapters are serialized into one buffer so the pretty printer sees the
	// separator that precedes them and puts each chapter's first block element
	// on a fresh line.
	var out strings.Builder
	for _, item := range trees {
		if out.Len() > 0 {
			out.WriteString(printSeparator)
		}
		html.SerializeInto(item.tree, &out)
	}
	return out.String()
}

// firstHeadingID returns the id of the chapter's first h1, if it has one.
func firstHeadingID(tree *html.Node) (string, bool) {
	for _, el := range tree.Elements(func(name string) bool { return name == "h1" }) {
		if id, ok := el.El.Attr("id"); ok {
			return id, true
		}
	}
	return "", false
}

// linkParts splits a link into scheme, path and anchor.
var linkParts = regexp.MustCompile(`^(?P<scheme>[a-z][a-z0-9+.-]*:)?(?P<path>[^#]+)?(?:#(?P<anchor>.*))?$`)

// rewritePrintLinks turns links that point at another chapter into in-page
// anchors, and leaves external links alone.
func rewritePrintLinks(tree *html.Node, selfPath string, remap map[string]map[string]string, rootIDs map[string]string) {
	rewrite := func(el *html.Element, attr string) {
		value, ok := el.Attr(attr)
		if !ok || value == "" {
			return
		}
		m := linkParts.FindStringSubmatch(value)
		if m == nil || m[1] != "" {
			// Absolute URL: leave untouched.
			return
		}
		target, anchor := m[2], m[3]
		if target == "" {
			// Same-page anchor: remap into the (possibly renamed) id.
			if renamed, ok := remap[selfPath][anchor]; ok {
				el.SetAttr(attr, "#"+renamed)
			}
			return
		}
		resolved := utils.NormalizePath(filepath.ToSlash(filepath.Join(filepath.Dir(selfPath), target)))
		table, isChapter := remap[resolved]
		if !isChapter {
			// Not a chapter: keep the relative path, which still resolves from
			// the print page at the book root.
			el.SetAttr(attr, resolved+anchorSuffix(anchor))
			return
		}
		if anchor == "" {
			el.SetAttr(attr, "#"+rootIDs[resolved])
			return
		}
		if renamed, ok := table[anchor]; ok {
			el.SetAttr(attr, "#"+renamed)
		} else {
			el.SetAttr(attr, "#"+anchor)
		}
	}

	for _, el := range tree.Elements(func(name string) bool { return name == "a" || name == "img" }) {
		switch el.El.Name {
		case "a":
			rewrite(el.El, "href")
		case "img":
			rewrite(el.El, "src")
		}
	}
}

func anchorSuffix(anchor string) string {
	if anchor == "" {
		return ""
	}
	return "#" + anchor
}

// emitRedirects writes one small HTML page per configured redirect. Ported from
// emit_redirects in crates/mdbook-html/src/html_handlebars/hbs_renderer.rs.
func emitRedirects(destination string, registry *hbs.Registry, redirects map[string]string) error {
	if len(redirects) == 0 {
		return nil
	}
	combined := combineFragmentRedirects(redirects)
	for _, original := range sortedMapKeys(combined) {
		entry := combined[original]
		target := filepath.Join(destination, filepath.FromSlash(strings.TrimPrefix(original, "/")))
		if _, err := os.Stat(target); err == nil {
			// A real page already lives here; the in-page fragment mapper
			// handles those redirects instead.
			continue
		}
		if entry.destination == "" {
			return fmt.Errorf("redirect entry for %q has no destination", original)
		}
		fragmentJSON, err := json.Marshal(entry.fragments)
		if err != nil {
			return err
		}
		page, err := registry.Render("redirect", map[string]any{
			"url":          entry.destination,
			"fragment_map": string(fragmentJSON),
		})
		if err != nil {
			return err
		}
		if err := utils.WriteFile(target, []byte(page)); err != nil {
			return err
		}
	}
	return nil
}

// redirectEntry groups a page's default destination with its per-fragment ones.
type redirectEntry struct {
	destination string
	fragments   map[string]string
}

// combineFragmentRedirects groups `page.html` and `page.html#frag` entries so
// each source page is emitted once.
func combineFragmentRedirects(redirects map[string]string) map[string]*redirectEntry {
	combined := map[string]*redirectEntry{}
	get := func(key string) *redirectEntry {
		if entry, ok := combined[key]; ok {
			return entry
		}
		entry := &redirectEntry{fragments: map[string]string{}}
		combined[key] = entry
		return entry
	}
	for source, target := range redirects {
		page, fragment, hasFragment := strings.Cut(source, "#")
		if hasFragment {
			get(page).fragments["#"+fragment] = target
			continue
		}
		get(page).destination = target
	}
	return combined
}

func sortedMapKeys[V any](m map[string]V) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}
