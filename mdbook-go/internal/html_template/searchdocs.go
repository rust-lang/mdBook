package html_template

import (
	"regexp"
	"strings"

	"mdbook-go/internal/html"
	"mdbook-go/internal/model"
	"mdbook-go/pkg/fs"
)

// collapseWhitespace squeezes runs of two or more whitespace characters into a
// single space, matching collapse_whitespace in
// crates/mdbook-html/src/html_handlebars/search.rs. A single newline is left
// alone, which is why chapter bodies keep their soft line breaks.
var multiWhitespace = regexp.MustCompile(`\s\s+`)

func collapseWhitespace(text string) string {
	return multiWhitespace.ReplaceAllString(text, " ")
}

// collectSearchDocs walks each chapter tree and splits it into one search
// document per heading, mirroring index_chapter in search.rs.
func collectSearchDocs(cfg model.Search, trees []*chapterTree) []Doc {
	var docs []Doc
	for _, item := range trees {
		if !chapterSearchEnabled(cfg, item.chapter.Path) {
			continue
		}
		docs = append(docs, indexChapter(cfg, item)...)
	}
	return docs
}

// chapterSearchEnabled applies the [output.html.search.chapter] overrides. The
// most specific matching path prefix wins.
func chapterSearchEnabled(cfg model.Search, path string) bool {
	enabled := true
	best := -1
	for prefix, settings := range cfg.Chapter {
		if path != prefix && !strings.HasPrefix(path, prefix+"/") {
			continue
		}
		if len(prefix) > best {
			best = len(prefix)
			enabled = settings.Enable
		}
	}
	return enabled
}

// indexChapter produces the search documents for a single chapter.
func indexChapter(cfg model.Search, item *chapterTree) []Doc {
	anchorBase := fs.ToURLPath(strings.TrimPrefix(item.chapter.HTMLPath(), "./"))
	breadcrumbs := append(append([]string{}, item.chapter.ParentNames...), item.chapter.Name)

	var (
		docs      []Doc
		inHeading bool
		sectionID string
		heading   strings.Builder
		body      strings.Builder
	)

	add := func(title, id string) {
		url := anchorBase
		if id != "" {
			url += "#" + id
		}
		docs = append(docs, Doc{
			URL:         url,
			Title:       collapseWhitespace(strings.TrimSpace(title)),
			Body:        collapseWhitespace(strings.TrimSpace(body.String())),
			Breadcrumbs: collapseWhitespace(strings.TrimSpace(strings.Join(breadcrumbs, " » "))),
		})
	}

	var visit func(n *html.Node)
	visit = func(n *html.Node) {
		if n.Kind == html.KindText {
			if inHeading {
				heading.WriteString(n.Text)
			} else {
				body.WriteString(n.Text)
			}
			return
		}
		if n.Kind == html.KindElement {
			level := headingLevel(n.El.Name)
			id, hasID := n.El.Attr("id")
			switch {
			case level > 0 && level <= cfg.HeadingSplitLevel && hasID:
				if heading.Len() > 0 {
					// A previous section just ended; flush it.
					add(heading.String(), sectionID)
					heading.Reset()
					body.Reset()
					breadcrumbs = breadcrumbs[:len(breadcrumbs)-1]
				}
				sectionID = id
				inHeading = true
			case n.El.Name == "script" || n.El.Name == "style":
				return
			case inHeading:
				heading.WriteByte(' ')
			default:
				body.WriteByte(' ')
			}
		}
		for _, c := range n.Children {
			visit(c)
		}
		if n.Kind == html.KindElement {
			if level := headingLevel(n.El.Name); level > 0 && level <= cfg.HeadingSplitLevel {
				inHeading = false
				breadcrumbs = append(breadcrumbs, heading.String())
			}
		}
	}
	visit(item.tree)

	if body.Len() > 0 || heading.Len() > 0 {
		title := heading.String()
		if title == "" && len(breadcrumbs) > 0 {
			title = breadcrumbs[0]
		}
		add(title, sectionID)
	}
	return docs
}

// headingLevel returns 1-6 for an h1..h6 element and 0 for anything else.
func headingLevel(name string) int {
	if len(name) == 2 && name[0] == 'h' && name[1] >= '1' && name[1] <= '6' {
		return int(name[1] - '0')
	}
	return 0
}
