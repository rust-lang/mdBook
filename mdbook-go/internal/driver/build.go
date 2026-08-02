package driver

import (
	"mdbook-go/internal/render"
)

// Build renders the book with the HTML backend. All of the work lives in
// internal/render; the driver only supplies the loaded book and the paths.
func (m *MDBook) Build() error {
	return render.Render(&render.Context{
		Root:        m.Root,
		Destination: m.BuildDir(),
		Config:      m.Config,
		Book:        m.Book,
	})
}
