// Package assets embeds the default mdBook front-end assets and the
// production Go templates.
//
// The css/js/searcher files under this directory are a verbatim copy of
// crates/mdbook-html/front-end/ in the Rust workspace; templates/ is Go-only
// (the html/template sources that replaced the Rust theme templates). They
// live here because Go's //go:embed directive cannot reach outside the
// module, and the Go port must be able to produce byte-identical output
// without the Rust tree being present. Refresh the front-end with:
//
//	rm -rf mdbook-go/internal/assets && mkdir -p mdbook-go/internal/assets \
//	  && cp -R crates/mdbook-html/front-end/. mdbook-go/internal/assets/
//
// (then restore this file and the templates/ dir, both Go-only with no Rust
// counterpart).
package static

import (
	"embed"
	"io/fs"
)

//go:embed all:css all:js all:searcher all:templates
var files embed.FS

// FS exposes the embedded front-end tree.
func FS() fs.FS {
	return files
}

// Templates returns the embedded production templates, sub-rooted at
// templates/ so callers can ReadDir("") to find the five .html files.
func Templates() fs.FS {
	sub, err := fs.Sub(files, "templates")
	if err != nil {
		panic("assets: templates sub: " + err.Error())
	}
	return sub
}

// MustRead returns the contents of an embedded file, panicking when the file is
// missing. Every call site uses a path that is present at compile time.
func MustRead(name string) []byte {
	data, err := files.ReadFile(name)
	if err != nil {
		panic("assets: " + err.Error())
	}
	return data
}
