// Package driver contains the MDBook orchestrator. This is the M1 minimum:
// it loads a book from disk, builds a flat HTML representation, and
// supports the init flow. Plugin processing, watch, serve and search are
// scheduled for later milestones.
package driver

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"mdbook-go/internal/book"
	"mdbook-go/internal/config"
	"mdbook-go/internal/summary"
)

// MDBook is the central handle for a loaded book.
type MDBook struct {
	Root   string
	Config *config.Config
	Book   *book.Book
}

// Load resolves the book root, reads book.toml and SUMMARY.md, and assembles
// the in-memory book tree.
func Load(root string) (*MDBook, error) {
	if root == "" {
		root = "."
	}
	abs, err := filepath.Abs(root)
	if err != nil {
		return nil, err
	}
	cfg, err := config.LoadBook(abs)
	if err != nil {
		return nil, err
	}
	if err := cfg.FromEnv(); err != nil {
		return nil, err
	}
	cfg.SetSourceDir(abs)

	m := &MDBook{Root: abs, Config: cfg, Book: book.NewBook()}
	if err := m.loadSummary(); err != nil {
		return nil, err
	}
	return m, nil
}

// loadSummary reads SUMMARY.md, converts each link into a Chapter and
// attaches nested children recursively.
//
// Only the numbered section is numbered, and the numbers are hierarchical:
// top-level entries get 1, 2, 3 and their children 1.1, 1.2 and so on. Part
// titles do not reset the counter, and draft chapters still consume a number —
// both verified against the Rust renderer's toc.html output.
func (m *MDBook) loadSummary() error {
	srcDir := m.Config.Book.SourceDir
	summaryPath := filepath.Join(srcDir, "SUMMARY.md")
	if _, err := os.Stat(summaryPath); err != nil {
		return fmt.Errorf("missing SUMMARY.md at %s", summaryPath)
	}
	sum, err := summary.ParseFile(summaryPath)
	if err != nil {
		return err
	}

	for _, item := range sum.PrefixChapters {
		if err := m.appendItem(item, srcDir, nil, nil); err != nil {
			return err
		}
	}
	counter := 0
	for _, item := range sum.NumberedChapters {
		if item.Link != nil {
			counter++
		}
		if err := m.appendItem(item, srcDir, nil, book.SectionNumber{uint(counter)}); err != nil {
			return err
		}
	}
	for _, item := range sum.SuffixChapters {
		if err := m.appendItem(item, srcDir, nil, nil); err != nil {
			return err
		}
	}
	return nil
}

// appendItem converts one summary item into a book item. number is nil for
// prefix and suffix chapters, which are not numbered.
func (m *MDBook) appendItem(item summary.SummaryItem, srcDir string, parents []string, number book.SectionNumber) error {
	switch {
	case item.PartTitle != nil:
		m.Book.Items = append(m.Book.Items, book.BookItem{PartTitle: &book.PartTitle{Name: item.PartTitle.Name}})
	case item.Separator != nil:
		m.Book.Items = append(m.Book.Items, book.BookItem{Separator: &book.Separator{}})
	case item.Link != nil:
		ch, err := m.newChapter(item.Link, srcDir, parents, number)
		if err != nil {
			return err
		}
		m.Book.Items = append(m.Book.Items, book.BookItem{Chapter: ch})
	}
	return nil
}

// newChapter builds a chapter and, recursively, its nested children.
func (m *MDBook) newChapter(link *summary.Link, srcDir string, parents []string, number book.SectionNumber) (*book.Chapter, error) {
	var ch *book.Chapter
	if link.Location == "" {
		ch = book.NewDraftChapter(link.Name)
	} else {
		content, err := readChapter(srcDir, link.Location)
		if err != nil {
			return nil, err
		}
		ch = book.NewChapter(link.Name, link.Location)
		ch.Content = content
		ch.SourcePath = link.Location
	}
	ch.Number = number
	ch.ParentNames = parents

	childParents := append(append([]string{}, parents...), link.Name)
	child := 0
	for _, nested := range link.NestedItems {
		switch {
		case nested.PartTitle != nil, nested.Separator != nil:
			// Only chapters can nest inside a chapter.
		case nested.Link != nil:
			var childNumber book.SectionNumber
			if number != nil {
				child++
				childNumber = append(append(book.SectionNumber{}, number...), uint(child))
			}
			sub, err := m.newChapter(nested.Link, srcDir, childParents, childNumber)
			if err != nil {
				return nil, err
			}
			ch.SubItems = append(ch.SubItems, book.BookItem{Chapter: sub})
		}
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
func (m *MDBook) SourceDir() string { return m.Config.Book.SourceDir }

// PathToRoot returns the relative path from a chapter's source location back
// to the book root.
func (m *MDBook) PathToRoot() string {
	srcRel, err := filepath.Rel(m.Root, m.Config.Book.SourceDir)
	if err != nil {
		return ""
	}
	depth := strings.Count(filepath.Clean(srcRel), string(os.PathSeparator))
	if depth == 0 {
		return "./"
	}
	return strings.Repeat("../", depth)
}
