// Package fontawesome reproduces, byte-for-byte, the SVG markup emitted by
// the Rust crate `font-awesome-as-a-crate` v0.3.1 that mdBook depends on
// (see /Users/qhai-dev/qhai-dev/mdBook/crates/mdbook-html/src/html_handlebars/helpers/fontawesome.rs).
//
// Only the icons mdBook can actually request through the `{{fa ...}}`
// handlebars helper are embedded here (see icons.go). To add more, drop the
// raw .svg contents from
//
//	font-awesome-as-a-crate-0.3.1/fontawesome-free-6.2.0-desktop/svgs/{type}/<name>.svg
//
// as a `const` in icons.go and register it in the appropriate `icons<Type>`
// map; the lookup happens at runtime.
//
// # Deprecated
//
// The Font Awesome feature will be removed in a future release of
// mdbook-go. The upstream mdBook Rust port is moving away from the
// embedded `font-awesome-as-a-crate` dependency; once that drops, the
// Go port will too. The 15 icons currently embedded cover only a small
// subset of Font Awesome Free 6.2.0, and growing that table to full
// parity (the harness diff against `tests/testsuite/rendering/fontawesome`
// reports 39 diff lines for the missing icons) costs ~700 KB of binary
// size for a feature the maintainers are phasing out.
//
// Users who depend on a specific icon should embed the SVG directly in
// their theme or via an `<img>` tag. The `{{fa ...}}` handlebars helper
// will continue to work until the package is removed; new code should
// not be written against this API.
package fontawesome

import (
	"errors"
	"fmt"
	"os"
	"strings"
	"sync"
)

// Type identifies the Font Awesome style for an icon.
//
// The numeric values are intentionally identical to the ones used by
// mdBook's Rust helper, in declaration order; do not reorder them.
type Type int

const (
	// Solid (aliased as "solid" or "fas" in templates).
	Solid Type = iota
	// Regular (aliased as "regular", "far", or plain "fa" — see TypeFromString).
	Regular
	// Brands (aliased as "brands" or "fab").
	Brands
)

// String returns the canonical lower-case name of the style, the same one
// the Rust crate uses for its `fontawesome_svg(dir, file)` lookup.
func (t Type) String() string {
	switch t {
	case Solid:
		return "solid"
	case Regular:
		return "regular"
	case Brands:
		return "brands"
	default:
		return ""
	}
}

// TypeFromString maps the strings accepted by mdBook's `{{fa ...}}` helper
// to a Type. The accepted inputs are:
//
//	solid, regular, brands      — explicit style names
//	fas, far, fab               — modern shorthand
//	fa                          — legacy alias for "regular"
//
// Anything else returns an error.
func TypeFromString(s string) (Type, error) {
	switch s {
	case "solid", "fas":
		return Solid, nil
	case "regular", "far", "fa":
		return Regular, nil
	case "brands", "fab":
		return Brands, nil
	default:
		return 0, fmt.Errorf(
			"fontawesome: invalid type %q (want solid|regular|brands, "+
				"fas|far|fab, or fa)",
			s,
		)
	}
}

// ErrUnknownIcon is the sentinel returned by SVG / Span when an icon name
// is not present in the embedded table.
var ErrUnknownIcon = errors.New("fontawesome: unknown icon")

// lookup returns the raw <svg>...</svg> body for (t, name) — what the
// `fontawesome_svg(dir, file)` function in font-awesome-as-a-crate 0.3.1
// returns. It is the byte-for-byte content of the upstream .svg file;
// no `<svg style=...>` rewrite, because that rewrite in build.rs only
// touches the `#[doc]` attribute, never the value returned to mdBook.
//
// If t is not one of the known Types or name is not in the icon table
// for that style (after stripping the `fa-`, `fab-`, `fas-` prefixes
// the handlebars helper accepts), an error is returned that is suitable
// for surfacing to the user (mirroring the message mdBook currently
// emits from its Rust helper).
var (
	deprecationOnce  sync.Once
	deprecationFired bool
)

// deprecationOnceDone reports whether warnDeprecated has fired. It
// exists only for the test that pins the once-only contract.
func deprecationOnceDone() bool { return deprecationFired }

// warnDeprecated emits a one-shot warning so users still using
// {{fa ...}} in their theme see a clear pointer to the planned
// removal. The warning goes to stderr so mdbook-go's other logging
// (which uses fmt.Fprintln(os.Stderr, …)) shares the same stream.
func warnDeprecated() {
	deprecationOnce.Do(func() {
		deprecationFired = true
		fmt.Fprintln(os.Stderr,
			"fontawesome: {{fa ...}} is deprecated and will be removed "+
				"in a future release; embed the SVG directly in your theme "+
				"or use <img> instead. See "+
				"mdbook-go/internal/fontawesome/fontawesome.go for details.")
	})
}

func lookup(t Type, name string) (string, error) {
	warnDeprecated()
	stripped := strings.TrimPrefix(name, "fab-")
	stripped = strings.TrimPrefix(stripped, "fas-")
	stripped = strings.TrimPrefix(stripped, "fa-")
	if stripped == "" || stripped != name && strings.Contains(stripped, "-") {
		// Don't bother validating further; just look up.
	}

	var table map[string]string
	switch t {
	case Solid:
		table = iconsSolid
	case Regular:
		table = iconsRegular
	case Brands:
		table = iconsBrands
	default:
		return "", fmt.Errorf("fontawesome: invalid type %d", t)
	}

	if svg, ok := table[stripped]; ok {
		return svg, nil
	}
	// Preserve the icon name the user passed so the error message reads
	// the same way mdBook's Rust helper writes it.
	userName := name
	if userName == "" {
		userName = stripped
	}
	return "", fmt.Errorf(
		"Unknown Font Awesome icon `%s` for type `%s`. "+
			"Hint: check the icon name and prefix (fas (solid), fab (brands), or far (regular)) "+
			"at https://fontawesome.com/v6/search?m=free",
		userName, t,
	)
}

// SVG returns the raw <svg ...>...</svg> string for the given style and
// icon name. The returned string is byte-identical to what the Rust crate
// `font-awesome-as-a-crate` v0.3.1 produces for the same inputs and is
// therefore suitable to drop straight into an mdBook HTML template.
//
// `name` may carry the prefixes mdBook's helper accepts ("fa-", "fab-",
// "fas-"); they are stripped before lookup.
func SVG(t Type, name string) (string, error) {
	return lookup(t, name)
}

// Span returns the wrapper element that mdBook's `{{fa ...}}` helper
// emits around an SVG icon:
//
//	<span class=fa-svg id="<id>">SVG</span>      when id != ""
//	<span class=fa-svg>SVG</span>                 when id == ""
//
// The SVG inside the wrapper is byte-identical to what the Rust crate
// would produce for the same (type, name) pair.
func Span(t Type, name, id string) (string, error) {
	svg, err := lookup(t, name)
	if err != nil {
		return "", err
	}
	if id != "" {
		return `<span class=fa-svg id="` + id + `">` + svg + `</span>`, nil
	}
	return `<span class=fa-svg>` + svg + `</span>`, nil
}
