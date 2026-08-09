// This file is a Go port of the elasticlunr index builder that mdBook uses
// (crates/mdbook-html/src/html_handlebars/search.rs), producing JSON output
// byte-identical to elasticlunr-rs v3.1.0. It implements the English-language
// Porter stemmer pipeline (trimmer, stopWordFilter, stemmer) and the
// inverted-index trie used by elasticlunr.js's on-disk search index format.
package html_template

import "strings"

// Options controls the search-index emission.
type Options struct {
	// LimitResults is the maximum number of search results returned to the
	// user. Default: 30.
	LimitResults int

	// TeaserWordCount is the number of words in the result teaser.
	// Default: 30.
	TeaserWordCount int

	// BoostTitle is the elasticlunr.js boost applied to the title field.
	// Default: 2.
	BoostTitle int

	// BoostParagraph is the elasticlunr.js boost applied to the body field.
	// Default: 1.
	BoostParagraph int

	// BoostHierarchy is the elasticlunr.js boost applied to the
	// breadcrumbs field. Default: 1.
	BoostHierarchy int

	// UseBooleanAnd switches the search-options boolean model between
	// "OR" (false, default) and "AND" (true).
	UseBooleanAnd bool

	// Expand enables the elasticlunr.js "expand" search option.
	// Default: true.
	Expand bool
}

// Doc is one document to index. Each Doc maps to one search entry: the URL is
// what the JS searcher links to, Title/Body/Breadcrumbs are the indexed text.
type Doc struct {
	URL         string
	Title       string
	Body        string
	Breadcrumbs string
}

// IndexJSON returns the JSON document that mdBook interpolates into
// `window.search = Object.assign(window.search, JSON.parse('...'));`.
//
// The returned string is byte-identical to what elasticlunr-rs v3.1.0 emits
// for the same inputs (modulo the fact that the float formatting follows
// serde_json/ryu's shortest-round-trip convention).
func IndexJSON(docs []Doc, opts Options) (string, error) {
	idx, docURLs := buildIndex(docs, opts)
	return serializeIndex(idx, docURLs, opts), nil
}

// JS returns the full contents of searchindex.js, i.e.
//
//	window.search = Object.assign(window.search, JSON.parse('...'));
//
// where the JSON payload has its `\` characters escaped to `\\` and its `'`
// characters escaped to `\'`, matching mdBook-html's search.rs.
func JS(docs []Doc, opts Options) (string, error) {
	jsonStr, err := IndexJSON(docs, opts)
	if err != nil {
		return "", err
	}
	// search.rs escapes `\` to `\\` first, then `'` to `\'`. The first
	// replacement doubles backslashes that come from JSON escaping, the second
	// escapes any literal apostrophes so the result is safe inside a JS
	// single-quoted string literal. The Rust format string has no trailing
	// newline, and the file is hashed, so neither may be added here.
	escaped := strings.ReplaceAll(jsonStr, `\`, `\\`)
	escaped = strings.ReplaceAll(escaped, `'`, `\'`)
	return "window.search = Object.assign(window.search, JSON.parse('" + escaped + "'));", nil
}

// buildIndex assembles the Index and docURLs for a given docs slice.
func buildIndex(docs []Doc, opts Options) (*Index, []string) {
	idx := newIndex()
	docURLs := make([]string, 0, len(docs))
	for i, d := range docs {
		docRef := formatInt(i)
		docURLs = append(docURLs, d.URL)
		idx.addDoc(docRef, d)
	}
	return idx, docURLs
}
