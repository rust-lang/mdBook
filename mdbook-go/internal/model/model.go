// Package model defines the in-memory book model and the typed/dynamic
// configuration objects that drive a build. The shapes mirror
// crates/mdbook-core/src/book.rs and config.rs so that JSON serialisation
// stays compatible with the existing preprocessor and renderer protocol.
package model

import (
	"fmt"
	"path/filepath"
	"strings"
)

// SectionNumber represents a hierarchical number like 1.2.3.
type SectionNumber []uint

// String renders the number with a trailing dot after every component, e.g.
// "1.2.3.". This matches SectionNumber's Display impl in
// crates/mdbook-core/src/book.rs, which the sidebar template relies on both for
// the visible label and for deriving the nesting level.
func (n SectionNumber) String() string {
	if len(n) == 0 {
		return ""
	}
	var b strings.Builder
	for _, v := range n {
		fmt.Fprintf(&b, "%d.", v)
	}
	return b.String()
}

// Chapter is one node in the book tree.
type Chapter struct {
	Name        string     `json:"name"`
	Content     string     `json:"content"`
	Number      SectionNum `json:"number,omitempty"`
	Path        string     `json:"path"`
	SourcePath  string     `json:"source_path"`
	ParentNames []string   `json:"parent_names"`
	SubItems    []BookItem `json:"sub_items"`
}

// SectionNum is a helper alias so the JSON tag matches the Rust field name.
type SectionNum = SectionNumber

// Separator is a non-rendering break inside the book list.
type Separator struct{}

// PartTitle is a heading for a group of chapters.
type PartTitle struct {
	Name string `json:"name"`
}

// BookItem is a tagged union of Chapter, Separator or PartTitle.
type BookItem struct {
	Chapter   *Chapter   `json:"chapter,omitempty"`
	Separator *Separator `json:"separator,omitempty"`
	PartTitle *PartTitle `json:"part_title,omitempty"`
}

// NewChapter constructs a Chapter with the given name and path. The source
// path is derived from the path when not explicitly set.
func NewChapter(name, path string) *Chapter {
	return &Chapter{
		Name:       name,
		Path:       path,
		SourcePath: path,
		SubItems:   []BookItem{},
	}
}

// NewDraftChapter creates a placeholder chapter that is not backed by a file.
func NewDraftChapter(name string) *Chapter {
	return &Chapter{
		Name:     name,
		Path:     "",
		SubItems: []BookItem{},
	}
}

// SubChapters returns only the child items that are real chapters.
func (c *Chapter) SubChapters() []*Chapter {
	var out []*Chapter
	for _, item := range c.SubItems {
		if item.Chapter != nil {
			out = append(out, item.Chapter)
		}
	}
	return out
}

// Book is the ordered list of top-level items.
type Book struct {
	Items []BookItem `json:"sections"`
}

// NewBook returns an empty Book.
func NewBook() *Book {
	return &Book{Items: []BookItem{}}
}

// Chapters returns all non-draft chapters in depth-first order.
func (b *Book) Chapters() []*Chapter {
	var out []*Chapter
	for _, item := range b.Items {
		if item.Chapter != nil {
			out = append(out, collectChapters(item.Chapter)...)
		}
	}
	return out
}

func collectChapters(c *Chapter) []*Chapter {
	out := []*Chapter{c}
	for _, item := range c.SubItems {
		if item.Chapter != nil {
			out = append(out, collectChapters(item.Chapter)...)
		}
	}
	return out
}

// Iter visits the book in depth-first order. The callback may return false to
// stop iteration.
func (b *Book) Iter(fn func(*Chapter) bool) {
	for _, item := range b.Items {
		if item.Chapter == nil {
			continue
		}
		if !walkChapter(item.Chapter, fn) {
			return
		}
	}
}

func walkChapter(c *Chapter, fn func(*Chapter) bool) bool {
	if !fn(c) {
		return false
	}
	for _, item := range c.SubItems {
		if item.Chapter == nil {
			continue
		}
		if !walkChapter(item.Chapter, fn) {
			return false
		}
	}
	return true
}

// IsDraft reports whether the chapter has no backing path.
func (c *Chapter) IsDraft() bool {
	return c.Path == ""
}

// HTMLPath returns the output file for a chapter, relative to the book root.
// The source path's directory structure is preserved and only the extension is
// replaced, so `guide/advanced/deep.md` becomes `guide/advanced/deep.html`.
func (c *Chapter) HTMLPath() string {
	if c.IsDraft() {
		return ""
	}
	base := strings.TrimSuffix(c.Path, filepath.Ext(c.Path))
	if base == "" {
		return ""
	}
	return base + ".html"
}
