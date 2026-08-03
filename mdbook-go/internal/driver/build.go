package driver

import (
	"fmt"

	"mdbook-go/internal/book"
	"mdbook-go/internal/config"
	"mdbook-go/internal/plugin"
	"mdbook-go/internal/render"
)

// htmlRendererName is the identifier mdBook uses for the built-in HTML
// backend. External renderers each get their own names.
const htmlRendererName = "html"

// Build renders the book with the HTML backend. Preprocessors run first,
// mirroring crates/mdbook-driver/src/mdbook.rs::MDBook::build. When no
// preprocessors are configured the book passes through unchanged and the
// ChapterTitles map stays nil.
func (m *MDBook) Build() error {
	pre, err := plugin.BuildPreprocessors(m.Config, m.Root)
	if err != nil {
		return err
	}
	processed, ctx, err := plugin.RunPreprocessors(pre, m.Config, m.Root, htmlRendererName, m.Book)
	if err != nil {
		return err
	}
	return render.Render(&render.Context{
		Root:          m.Root,
		Destination:   m.BuildDir(),
		Config:        m.Config,
		Book:          processed,
		ChapterTitles: ctx.ChapterTitles,
	})
}

// PreprocessBook runs only the preprocessor chain and returns the resulting
// book and the populated PreprocessorContext. Tests and tools that want to
// inspect a preprocessed book without rendering use this entry point.
func (m *MDBook) PreprocessBook() (*book.Book, *plugin.PreprocessorContext, error) {
	pre, err := plugin.BuildPreprocessors(m.Config, m.Root)
	if err != nil {
		return nil, nil, err
	}
	return plugin.RunPreprocessors(pre, m.Config, m.Root, htmlRendererName, m.Book)
}

// RenderForBackend runs the preprocessor chain then renders via the chosen
// backend. When name == htmlRendererName the in-process HTML renderer is
// used; any other name dispatches to a CmdRenderer registered in the config.
func (m *MDBook) RenderForBackend(name string) error {
	if name != htmlRendererName {
		return fmt.Errorf("only the html backend is implemented in M3; got %q", name)
	}
	return m.Build()
}

// ensureConfigLoaded returns a non-nil config; cmd entry points that accept
// an optional config use it.
func ensureConfigLoaded(c *config.Config) *config.Config {
	if c == nil {
		c = config.New()
	}
	return c
}