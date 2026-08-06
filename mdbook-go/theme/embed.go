// Package themedata embeds the default mdBook front-end assets.
//
// The files under this directory are a verbatim copy of
// crates/mdbook-html/front-end/ in the Rust workspace. They live here because
// Go's //go:embed directive cannot reach outside the module, and the Go port
// must be able to produce byte-identical output without the Rust tree being
// present. Refresh them with:
//
//	rm -rf mdbook-go/theme && mkdir -p mdbook-go/theme \
//	  && cp -R crates/mdbook-html/front-end/. mdbook-go/theme/
//
// (then restore this file, which is Go-only and has no Rust counterpart).
package themedata

import (
	"embed"
	"io/fs"
)

//go:embed all:css all:fonts all:images all:js all:searcher all:templates
var files embed.FS

// FS exposes the embedded front-end tree.
func FS() fs.FS {
	return files
}

// MustRead returns the contents of an embedded file, panicking when the file is
// missing. Every call site uses a path that is present at compile time.
func MustRead(name string) []byte {
	data, err := files.ReadFile(name)
	if err != nil {
		panic("themedata: " + err.Error())
	}
	return data
}
