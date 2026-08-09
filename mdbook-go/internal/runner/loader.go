// Package runner drives the book build pipeline: it loads a book from
// disk (config + its [chapters] section), assembles the in-memory book tree, and
// exposes the build/init entry points used by the CLI. It is the Go
// counterpart of crates/mdbook-driver (see docs/crate-mapping.md §2 and
// docs/runner-vs-rust.md).
package runner

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"mdbook-go/internal/model"
)

// MDBook is the central handle for a loaded book.
type MDBook struct {
	Root   string
	Config *model.Config
	Book   *model.Book
}

// Load resolves the book root, reads doclens.yaml, and assembles the
// in-memory book tree from its [chapters] section.
func Load(root string) (*MDBook, error) {
	if root == "" {
		root = "."
	}
	abs, err := filepath.Abs(root)
	if err != nil {
		return nil, err
	}
	cfg, err := model.LoadBook(abs)
	if err != nil {
		return nil, err
	}
	cfg.SetRoot(abs)

	m := &MDBook{Root: abs, Config: cfg, Book: model.NewBook()}
	if err := m.loadChapters(); err != nil {
		return nil, err
	}
	return m, nil
}

// loadChapters reads the [chapters] section of doclens.yaml, converts each
// entry into a Chapter and attaches nested children recursively.
//
// Only the numbered section is numbered, and the numbers are hierarchical:
// top-level entries get 1, 2, 3 and their children 1.1, 1.2 and so on. Part
// titles do not reset the counter, and draft chapters still consume a number —
// both verified against the Rust renderer's toc.html output.
func (m *MDBook) loadChapters() error {
	ch := m.Config.Chapters
	for _, item := range ch.Prefix {
		if err := m.appendItem(item, nil, nil); err != nil {
			return err
		}
	}
	counter := 0
	for _, item := range ch.Numbered {
		switch {
		case item.Separator, item.Part != "", item.Name == "" && item.Path == "":
			// Separators, part titles and empty entries consume no number.
		default:
			counter++
		}
		if err := m.appendItem(item, nil, model.SectionNumber{uint(counter)}); err != nil {
			return err
		}
	}
	for _, item := range ch.Suffix {
		if err := m.appendItem(item, nil, nil); err != nil {
			return err
		}
	}
	return nil
}

// appendItem converts one chapter-list entry into a book item. number is nil
// for prefix and suffix chapters, which are not numbered.
func (m *MDBook) appendItem(item model.ChapterItem, parents []string, number model.SectionNumber) error {
	switch {
	case item.Separator:
		m.Book.Items = append(m.Book.Items, model.BookItem{Separator: &model.Separator{}})
	case item.Part != "":
		m.Book.Items = append(m.Book.Items, model.BookItem{PartTitle: &model.PartTitle{Name: item.Part}})
	case item.Name != "" || item.Path != "":
		ch, err := m.newChapter(item, parents, number)
		if err != nil {
			return err
		}
		m.Book.Items = append(m.Book.Items, model.BookItem{Chapter: ch})
	}
	return nil
}

// newChapter builds a chapter and, recursively, its nested children.
func (m *MDBook) newChapter(item model.ChapterItem, parents []string, number model.SectionNumber) (*model.Chapter, error) {
	// Strip leading "./" so chapter Path stays "comment-in-list.md" rather
	// than "./comment-in-list.md" — matches Rust's SUMMARY parser.
	path := strings.TrimPrefix(item.Path, "./")
	var ch *model.Chapter
	if path == "" {
		ch = model.NewDraftChapter(item.Name)
	} else {
		content, err := readChapter(m.SourceDir(), path)
		if err != nil {
			return nil, err
		}
		ch = model.NewChapter(item.Name, path)
		ch.Content = content
		ch.SourcePath = path
	}
	ch.Number = number
	ch.ParentNames = parents

	childParents := append(append([]string{}, parents...), item.Name)
	child := 0
	for _, nested := range item.Children {
		if nested.Separator || nested.Part != "" || (nested.Name == "" && nested.Path == "") {
			// Only chapters can nest inside a chapter.
			continue
		}
		var childNumber model.SectionNumber
		if number != nil {
			child++
			childNumber = append(append(model.SectionNumber{}, number...), uint(child))
		}
		sub, err := m.newChapter(nested, childParents, childNumber)
		if err != nil {
			return nil, err
		}
		ch.SubItems = append(ch.SubItems, model.BookItem{Chapter: sub})
	}
	return ch, nil
}

func readChapter(srcDir, location string) (string, error) {
	full := filepath.Join(srcDir, location)
	data, err := os.ReadFile(full)
	if err != nil {
		return "", fmt.Errorf("read chapter %s: %w", full, err)
	}
	// Strip UTF-8 BOM if present.
	data = bytes.TrimPrefix(data, []byte{0xEF, 0xBB, 0xBF})
	return string(data), nil
}

// BuildDir returns the absolute output directory.
func (m *MDBook) BuildDir() string {
	bd := m.Config.Build.BuildDir
	if !filepath.IsAbs(bd) {
		bd = filepath.Join(m.Root, bd)
	}
	return bd
}

// SourceDir returns the absolute source directory.
func (m *MDBook) SourceDir() string { return m.Config.Package.Root }

// PathToRoot returns the relative path from a chapter's source location back
// to the book root.
func (m *MDBook) PathToRoot() string {
	srcRel, err := filepath.Rel(m.Root, m.Config.Package.Root)
	if err != nil {
		return ""
	}
	depth := strings.Count(filepath.Clean(srcRel), string(os.PathSeparator))
	if depth == 0 {
		return "./"
	}
	return strings.Repeat("../", depth)
}
