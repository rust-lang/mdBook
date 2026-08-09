// Package plugin implements mdBook's preprocessor and renderer extension
// protocol. The JSON shapes in this file mirror what the Rust mdBook
// implementation serialises over stdin/stdout (see
// crates/mdbook-preprocessor/src/lib.rs and crates/mdbook-renderer/src/lib.rs).
// Keeping this wire format separate from the internal book model lets the
// latter evolve without breaking plugin compatibility.
package plugin

import (
	"encoding/json"

	"mdbook-go/internal/model"
)

// WireBook is the externally visible form of a book. It is serialised as the
// second element of the preprocessor input tuple and as the standalone
// preprocessor output.
//
// Field order and JSON tags are chosen so encoding/json produces the same byte
// stream serde does for crates/mdbook-core/src/book.rs::Book.
type WireBook struct {
	Items []WireBookItem `json:"items"`
}

// WireBookItem is the externally-tagged enum produced by serde for
// `enum BookItem { Chapter(Chapter), Separator, PartTitle(String) }`. The
// three cases are encoded as:
//
//	{"Chapter": {...}}      // tuple variant with named struct payload
//	null                    // unit variant
//	{"PartTitle": "name"}   // newtype variant
type WireBookItem struct {
	// Only one of the following is non-nil at a time. MarshalJSON below
	// produces the right externally-tagged shape and UnmarshalJSON
	// dispatches on which key is present.
	Chapter   *WireChapter `json:"-"`
	Separator *struct{}    `json:"-"`
	PartTitle *string      `json:"-"`
}

// MarshalJSON renders WireBookItem as the externally tagged enum.
func (b WireBookItem) MarshalJSON() ([]byte, error) {
	switch {
	case b.Chapter != nil:
		return json.Marshal(struct {
			Chapter *WireChapter `json:"Chapter"`
		}{b.Chapter})
	case b.PartTitle != nil:
		return json.Marshal(struct {
			PartTitle string `json:"PartTitle"`
		}{*b.PartTitle})
	default:
		// Unit variant (Separator) renders as JSON null.
		return []byte("null"), nil
	}
}

// UnmarshalJSON is the inverse: dispatch on which key is present.
func (b *WireBookItem) UnmarshalJSON(data []byte) error {
	if string(data) == "null" {
		b.Separator = &struct{}{}
		return nil
	}
	var probe struct {
		Chapter   *WireChapter `json:"Chapter"`
		Separator *struct{}    `json:"Separator"`
		PartTitle *string      `json:"PartTitle"`
	}
	if err := json.Unmarshal(data, &probe); err != nil {
		return err
	}
	switch {
	case probe.Chapter != nil:
		b.Chapter = probe.Chapter
	case probe.PartTitle != nil:
		b.PartTitle = probe.PartTitle
	default:
		if probe.Separator == nil {
			probe.Separator = &struct{}{}
		}
		b.Separator = probe.Separator
	}
	return nil
}

// WireChapter mirrors crates/mdbook-core/src/book.rs::Chapter.
type WireChapter struct {
	Name        string          `json:"name"`
	Content     string          `json:"content"`
	Number      *WireSectionNum `json:"number,omitempty"`
	SubItems    []WireBookItem  `json:"sub_items"`
	Path        *string         `json:"path"`        // Option<PathBuf>
	SourcePath  *string         `json:"source_path"` // Option<PathBuf>
	ParentNames []string        `json:"parent_names"`
}

// WireSectionNum mirrors `Option<SectionNumber>` so it can serialise as null
// or an array of integers. Section numbers in SUMMARY.md are 1-based and fit
// easily in 32 bits; using uint32 keeps the JSON wire form identical to
// serde's u32 representation.
type WireSectionNum []uint32

// WireConfig mirrors crates/mdbook-core/src/config.rs::Config. Field tags are
// snake_case to match serde's defaults; the dynamic `output` and
// `preprocessor` maps stay as raw JSON so nested tables remain opaque to
// plugins.
type WireConfig struct {
	Package      PackageConfig  `json:"book"`
	Build        BuildConfig    `json:"build"`
	Output       map[string]any `json:"output"`
	Preprocessor map[string]any `json:"preprocessor"`
}

// PackageConfig mirrors mdbook-core's BookConfig.
type PackageConfig struct {
	Title         string `json:"title"`
	Description   string `json:"description"`
	Language      string `json:"language"`
	TextDirection string `json:"text-direction"`
	Root          string `json:"src"`
}

// BuildConfig mirrors mdbook-core's BuildConfig.
type BuildConfig struct {
	BuildDir                string   `json:"build-dir"`
	ExtraWatchDirs          []string `json:"extra-watch-dirs"`
	CreateMissing           bool     `json:"create-missing"`
	UseDefaultPreprocessors bool     `json:"use-default-preprocessors"`
}

// WirePreprocessorContext is the JSON handed to an external preprocessor on
// stdin. See crates/mdbook-preprocessor/src/lib.rs::PreprocessorContext.
type WirePreprocessorContext struct {
	Root          string     `json:"root"`
	Config        WireConfig `json:"config"`
	Renderer      string     `json:"renderer"`
	MdbookVersion string     `json:"mdbook_version"`
	// chapter_titles is internal to mdBook and is skipped on the wire (the
	// Rust side marks it with #[serde(skip)]).
	ChapterTitles map[string]string `json:"-"`
}

// WireRenderContext is the JSON handed to an external renderer. See
// crates/mdbook-renderer/src/lib.rs::RenderContext.
type WireRenderContext struct {
	Version     string     `json:"version"`
	Root        string     `json:"root"`
	Book        WireBook   `json:"book"`
	Config      WireConfig `json:"config"`
	Destination string     `json:"destination"`
	// chapter_titles is internal and skipped.
	ChapterTitles map[string]string `json:"-"`
}

// MdbookVersion is the version string embedded in both context types. The
// Rust side reads CARGO_PKG_VERSION; the Go side stamps whatever value is
// passed to Build.
const MdbookVersion = "0.1.0-m3"

// ToWireBook converts the internal book model into the wire representation.
func ToWireBook(b *model.Book) WireBook {
	if b == nil {
		return WireBook{}
	}
	out := WireBook{Items: make([]WireBookItem, 0, len(b.Items))}
	for _, it := range b.Items {
		out.Items = append(out.Items, toWireItem(it))
	}
	return out
}

func toWireItem(it model.BookItem) WireBookItem {
	switch {
	case it.Chapter != nil:
		return WireBookItem{Chapter: toWireChapter(it.Chapter)}
	case it.PartTitle != nil:
		name := it.PartTitle.Name
		return WireBookItem{PartTitle: &name}
	default:
		return WireBookItem{Separator: &struct{}{}}
	}
}

func toWireChapter(c *model.Chapter) *WireChapter {
	wc := &WireChapter{
		Name:        c.Name,
		Content:     c.Content,
		SubItems:    make([]WireBookItem, 0, len(c.SubItems)),
		ParentNames: append([]string(nil), c.ParentNames...),
	}
	if c.IsDraft() {
		wc.Path = nil
		wc.SourcePath = nil
	} else {
		p := c.Path
		sp := c.SourcePath
		wc.Path = &p
		wc.SourcePath = &sp
	}
	if len(c.Number) > 0 {
		wn := make(WireSectionNum, 0, len(c.Number))
		for _, v := range c.Number {
			wn = append(wn, uint32(v))
		}
		wc.Number = &wn
	}
	for _, sub := range c.SubItems {
		wc.SubItems = append(wc.SubItems, toWireItem(sub))
	}
	return wc
}

// FromWireBook turns a wire representation back into the internal model. It
// is used after an external preprocessor has returned its modified book.
func FromWireBook(w WireBook) *model.Book {
	out := model.NewBook()
	for _, it := range w.Items {
		out.Items = append(out.Items, fromWireItem(it))
	}
	return out
}

func fromWireItem(it WireBookItem) model.BookItem {
	switch {
	case it.Chapter != nil:
		return model.BookItem{Chapter: fromWireChapter(it.Chapter)}
	case it.PartTitle != nil:
		return model.BookItem{PartTitle: &model.PartTitle{Name: *it.PartTitle}}
	default:
		return model.BookItem{Separator: &model.Separator{}}
	}
}

func fromWireChapter(wc *WireChapter) *model.Chapter {
	c := model.NewChapter(wc.Name, "")
	c.Content = wc.Content
	c.ParentNames = append([]string(nil), wc.ParentNames...)
	if wc.Path != nil {
		c.Path = *wc.Path
	}
	if wc.SourcePath != nil {
		c.SourcePath = *wc.SourcePath
	}
	if wc.Number != nil {
		sn := make(model.SectionNumber, 0, len(*wc.Number))
		for _, v := range *wc.Number {
			sn = append(sn, uint(v))
		}
		c.Number = sn
	}
	for _, sub := range wc.SubItems {
		c.SubItems = append(c.SubItems, fromWireItem(sub))
	}
	return c
}

// ToWireConfig converts the internal config to the wire shape.
func ToWireConfig(c *model.Config) WireConfig {
	if c == nil {
		return WireConfig{}
	}
	return WireConfig{
		Package: PackageConfig{
			Title:         c.Package.Title,
			Description:   c.Package.Description,
			Language:      c.Package.Language,
			TextDirection: c.Package.TextDirection,
			Root:          c.Package.Root,
		},
		Build: BuildConfig{
			BuildDir:                c.Build.BuildDir,
			ExtraWatchDirs:          c.Build.ExtraWatchDirs,
			CreateMissing:           c.Build.CreateMissing,
			UseDefaultPreprocessors: c.Build.UseDefaultPreprocessors,
		},
		Output:       c.Output,
		Preprocessor: c.Preprocessor,
	}
}
