package render

import (
	"encoding/json"
	"fmt"
	"html/template"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"mdbook-go/internal/html"
	"mdbook-go/internal/tplgotpl"
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
	remap := make(map[string]map[string]string, len(trees))
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

	var out strings.Builder
	for _, item := range trees {
		if out.Len() > 0 {
			out.WriteString(printSeparator)
		}
		html.SerializeInto(item.tree, &out)
	}
	return out.String()
}

func firstHeadingID(tree *html.Node) (string, bool) {
	for _, el := range tree.Elements(func(name string) bool { return name == "h1" }) {
		if id, ok := el.El.Attr("id"); ok {
			return id, true
		}
	}
	return "", false
}

var linkParts = regexp.MustCompile(`^(?P<scheme>[a-z][a-z0-9+.-]*:)?(?P<path>[^#]+)?(?:#(?P<anchor>.*))?$`)

func rewritePrintLinks(tree *html.Node, selfPath string, remap map[string]map[string]string, rootIDs map[string]string) {
	rewrite := func(el *html.Element, attr string) {
		value, ok := el.Attr(attr)
		if !ok || value == "" {
			return
		}
		m := linkParts.FindStringSubmatch(value)
		if m == nil || m[1] != "" {
			return
		}
		target, anchor := m[2], m[3]
		if target == "" {
			if renamed, ok := remap[selfPath][anchor]; ok {
				el.SetAttr(attr, "#"+renamed)
			}
			return
		}
		resolved := utils.NormalizePath(filepath.ToSlash(filepath.Join(filepath.Dir(selfPath), target)))
		table, isChapter := remap[resolved]
		if !isChapter {
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

// emitRedirects writes one small HTML page per configured redirect.
func emitRedirects(destination string, registry *tplgotpl.Registry, redirects map[string]string) error {
	if len(redirects) == 0 {
		return nil
	}
	combined := combineFragmentRedirects(redirects)
	for _, original := range sortedMapKeys(combined) {
		entry := combined[original]
		target := filepath.Join(destination, filepath.FromSlash(strings.TrimPrefix(original, "/")))
		if _, err := os.Stat(target); err == nil {
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
			"url":          template.URL(entry.destination),
			"fragment_map": template.JS(string(fragmentJSON)),
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

type redirectEntry struct {
	destination string
	fragments   map[string]string
}

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
