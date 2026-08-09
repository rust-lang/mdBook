package plugin

import (
	"mdbook-go/internal/model"
)

// Preprocessor mirrors crates/mdbook-preprocessor/src/lib.rs::Preprocessor.
//
// Implementations can be in-process (LinkPreprocessor, IndexPreprocessor) or
// out-of-process command shells (CmdPreprocessor).
type Preprocessor interface {
	// Name returns the registered name as it appears in doclens.yaml.
	Name() string

	// Run transforms the book according to the preprocessor's semantics.
	Run(ctx *PreprocessorContext, b *model.Book) (*model.Book, error)

	// SupportsRenderer reports whether the preprocessor is compatible with a
	// given renderer name. Built-in preprocessors default to true; custom
	// command preprocessors probe via "<cmd> supports <renderer>".
	SupportsRenderer(renderer string) (bool, error)
}

// PreprocessorContext carries the inputs and bookkeeping for one
// preprocessor invocation. ChapterTitles is mutable: preprocessors may record
// per-chapter title overrides here (e.g. {{#title ...}}).
type PreprocessorContext struct {
	// Root is the absolute book root directory.
	Root string
	// Config is the effective book configuration.
	Config *PreprocessorConfig
	// Renderer is the name of the renderer being prepared for.
	Renderer string
	// MdbookVersion is the version string stamped into the wire context.
	MdbookVersion string
	// ChapterTitles records overrides set by preprocessors like `links`.
	ChapterTitles map[string]string
}

// PreprocessorConfig is the read-only subset of *model.Config that
// preprocessors are likely to inspect. It keeps the protocol decoupled from
// the typed config package so other callers (e.g. tests) can supply their own.
type PreprocessorConfig struct {
	Package      PackageConfig
	Build        BuildConfig
	Output       map[string]any
	Preprocessor map[string]any
}

// NewPreprocessorConfig builds a PreprocessorConfig from a WireConfig
// (typically the value that came in over the wire).
func NewPreprocessorConfig(wc WireConfig) *PreprocessorConfig {
	return &PreprocessorConfig{
		Package:      wc.Package,
		Build:        wc.Build,
		Output:       wc.Output,
		Preprocessor: wc.Preprocessor,
	}
}

// Renderer mirrors crates/mdbook-renderer/src/lib.rs::Renderer.
type Renderer interface {
	Name() string
	// Render produces artefacts under ctx.Destination.
	Render(ctx *RenderContext) error
}

// RenderContext is what the renderer is handed.
type RenderContext struct {
	// Root is the book root.
	Root string
	// Book is the post-preprocessing book.
	Book *model.Book
	// Config is the effective book configuration.
	Config *PreprocessorConfig
	// Destination is the directory the renderer must write to.
	Destination string
	// MdbookVersion is the version string of the calling mdBook.
	MdbookVersion string
	// ChapterTitles is populated from the preprocessor run for use by
	// renderers that consult per-chapter title overrides.
	ChapterTitles map[string]string
}

// ToWirePreprocessorContext converts an in-process context into its wire form.
func ToWirePreprocessorContext(c *PreprocessorContext) WirePreprocessorContext {
	if c == nil {
		return WirePreprocessorContext{MdbookVersion: MdbookVersion}
	}
	wc := WirePreprocessorConfig(c.Config)
	return WirePreprocessorContext{
		Root:          c.Root,
		Config:        wc,
		Renderer:      c.Renderer,
		MdbookVersion: c.MdbookVersion,
		ChapterTitles: c.ChapterTitles,
	}
}

// FromWirePreprocessorContext is the inverse.
func FromWirePreprocessorContext(w WirePreprocessorContext) *PreprocessorContext {
	return &PreprocessorContext{
		Root:          w.Root,
		Config:        NewPreprocessorConfig(w.Config),
		Renderer:      w.Renderer,
		MdbookVersion: w.MdbookVersion,
		ChapterTitles: w.ChapterTitles,
	}
}

// WirePreprocessorConfig lifts a *PreprocessorConfig to the wire shape.
func WirePreprocessorConfig(c *PreprocessorConfig) WireConfig {
	if c == nil {
		return WireConfig{}
	}
	return WireConfig{
		Package:      c.Package,
		Build:        c.Build,
		Output:       c.Output,
		Preprocessor: c.Preprocessor,
	}
}

// ToWireRenderContext lifts a *RenderContext to its wire form.
func ToWireRenderContext(c *RenderContext) WireRenderContext {
	if c == nil {
		return WireRenderContext{Version: MdbookVersion}
	}
	return WireRenderContext{
		Version:       c.MdbookVersion,
		Root:          c.Root,
		Book:          ToWireBook(c.Book),
		Config:        WirePreprocessorConfig(c.Config),
		Destination:   c.Destination,
		ChapterTitles: c.ChapterTitles,
	}
}
