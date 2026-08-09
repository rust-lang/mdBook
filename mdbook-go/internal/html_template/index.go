package html_template

import (
	"math"
	"sort"
	"strconv"
	"strings"
)

// Index is the elasticlunr index under construction. The field order and the
// nesting shape are dictated by elasticlunr-rs v3.1.0's serde output, which the
// JSON writer below reproduces exactly.
type Index struct {
	// fields is the ordered field list: title, body, breadcrumbs.
	fields []string
	// trees holds one inverted-index trie per field.
	trees map[string]*indexItem
	// docs maps a document ref to its stored fields.
	docs   []storedDoc
	docIDs []string
}

// storedDoc is one entry of documentStore.docs plus its documentStore.docInfo
// token counts.
type storedDoc struct {
	ref    string
	fields map[string]string
	counts map[string]int
}

// indexItem is one trie node: the documents whose token ends here, the document
// frequency, and the child nodes keyed by the next character.
type indexItem struct {
	docs     map[string]float64
	df       int
	children map[string]*indexItem
}

func newIndexItem() *indexItem {
	return &indexItem{docs: map[string]float64{}, children: map[string]*indexItem{}}
}

// newIndex returns an empty index over the three fields mdBook uses.
func newIndex() *Index {
	fields := []string{"title", "body", "breadcrumbs"}
	trees := make(map[string]*indexItem, len(fields))
	for _, f := range fields {
		trees[f] = newIndexItem()
	}
	return &Index{fields: fields, trees: trees}
}

// formatInt renders a document reference. Documents are referenced by their
// ordinal, stringified.
func formatInt(i int) string { return strconv.Itoa(i) }

// tokenize splits a field value into search tokens, matching the `tokenize`
// helper in crates/mdbook-html/src/html_handlebars/search.rs: split on
// whitespace and `-`, lowercase, and drop anything longer than 80 characters.
func tokenize(text string) []string {
	fields := strings.FieldsFunc(text, func(r rune) bool {
		return r == '-' || r == ' ' || r == '\t' || r == '\n' || r == '\r' ||
			r == '\v' || r == '\f'
	})
	out := make([]string, 0, len(fields))
	for _, token := range fields {
		lower := strings.ToLower(token)
		if len([]rune(lower)) > 80 {
			continue
		}
		out = append(out, lower)
	}
	return out
}

// addDoc indexes one document under the given reference.
func (idx *Index) addDoc(ref string, d Doc) {
	values := map[string]string{
		"title":       d.Title,
		"body":        d.Body,
		"breadcrumbs": d.Breadcrumbs,
	}
	stored := storedDoc{
		ref:    ref,
		fields: map[string]string{"id": ref},
		counts: map[string]int{},
	}
	for field, value := range values {
		stored.fields[field] = value
	}

	for _, field := range idx.fields {
		tokens := runEnglishPipeline(tokenize(values[field]))
		stored.counts[field] = len(tokens)

		// elasticlunr scores a term by the square root of how often it occurs
		// in the field.
		frequency := map[string]int{}
		for _, token := range tokens {
			frequency[token]++
		}
		for token, count := range frequency {
			idx.trees[field].addToken(token, ref, math.Sqrt(float64(count)))
		}
	}
	idx.docs = append(idx.docs, stored)
	idx.docIDs = append(idx.docIDs, ref)
}

// addToken walks the trie one character at a time, creating nodes as needed,
// and records the term frequency on the final node.
func (item *indexItem) addToken(token, ref string, tf float64) {
	node := item
	for _, ch := range token {
		key := string(ch)
		child, ok := node.children[key]
		if !ok {
			child = newIndexItem()
			node.children[key] = child
		}
		node = child
	}
	if _, seen := node.docs[ref]; !seen {
		node.df++
	}
	node.docs[ref] = tf
}

// jsonWriter builds JSON with explicit key ordering, which encoding/json cannot
// do for maps. Object keys are emitted in the order elasticlunr-rs emits them:
// struct fields in declaration order, maps sorted by key.
type jsonWriter struct{ b strings.Builder }

func (w *jsonWriter) raw(s string)    { w.b.WriteString(s) }
func (w *jsonWriter) key(name string) { w.str(name); w.b.WriteByte(':') }
func (w *jsonWriter) comma()          { w.b.WriteByte(',') }
func (w *jsonWriter) openObject()     { w.b.WriteByte('{') }
func (w *jsonWriter) closeObject()    { w.b.WriteByte('}') }
func (w *jsonWriter) openArray()      { w.b.WriteByte('[') }
func (w *jsonWriter) closeArray()     { w.b.WriteByte(']') }
func (w *jsonWriter) int(v int)       { w.b.WriteString(strconv.Itoa(v)) }
func (w *jsonWriter) bool(v bool)     { w.b.WriteString(strconv.FormatBool(v)) }
func (w *jsonWriter) String() string  { return w.b.String() }

// float writes a number the way serde_json does: the shortest representation
// that round-trips, always carrying a decimal point for whole values.
func (w *jsonWriter) float(v float64) {
	s := strconv.FormatFloat(v, 'g', -1, 64)
	if !strings.ContainsAny(s, ".eE") {
		s += ".0"
	}
	w.b.WriteString(s)
}

// str writes a JSON string using serde_json's escaping rules: quotes and
// backslashes, the C0 control characters with short escapes where they exist,
// and \u00XX otherwise. Non-ASCII characters are emitted literally.
func (w *jsonWriter) str(s string) {
	w.b.WriteByte('"')
	for _, r := range s {
		switch r {
		case '"':
			w.b.WriteString(`\"`)
		case '\\':
			w.b.WriteString(`\\`)
		case '\n':
			w.b.WriteString(`\n`)
		case '\r':
			w.b.WriteString(`\r`)
		case '\t':
			w.b.WriteString(`\t`)
		case '\b':
			w.b.WriteString(`\b`)
		case '\f':
			w.b.WriteString(`\f`)
		default:
			if r < 0x20 {
				w.b.WriteString(`\u00`)
				const hex = "0123456789abcdef"
				w.b.WriteByte(hex[r>>4])
				w.b.WriteByte(hex[r&0xf])
				continue
			}
			w.b.WriteRune(r)
		}
	}
	w.b.WriteByte('"')
}

// serializeIndex renders the whole search index document.
func serializeIndex(idx *Index, docURLs []string, opts Options) string {
	w := &jsonWriter{}
	w.openObject()

	w.key("results_options")
	w.openObject()
	w.key("limit_results")
	w.int(opts.LimitResults)
	w.comma()
	w.key("teaser_word_count")
	w.int(opts.TeaserWordCount)
	w.closeObject()
	w.comma()

	w.key("search_options")
	w.openObject()
	w.key("bool")
	if opts.UseBooleanAnd {
		w.str("AND")
	} else {
		w.str("OR")
	}
	w.comma()
	w.key("expand")
	w.bool(opts.Expand)
	w.comma()
	w.key("fields")
	w.openObject()
	// Boost keys are a BTreeMap, so they come out alphabetically.
	w.key("body")
	w.openObject()
	w.key("boost")
	w.int(opts.BoostParagraph)
	w.closeObject()
	w.comma()
	w.key("breadcrumbs")
	w.openObject()
	w.key("boost")
	w.int(opts.BoostHierarchy)
	w.closeObject()
	w.comma()
	w.key("title")
	w.openObject()
	w.key("boost")
	w.int(opts.BoostTitle)
	w.closeObject()
	w.closeObject()
	w.closeObject()
	w.comma()

	w.key("doc_urls")
	w.openArray()
	for i, url := range docURLs {
		if i > 0 {
			w.comma()
		}
		w.str(url)
	}
	w.closeArray()
	w.comma()

	w.key("index")
	writeElasticlunrIndex(w, idx)

	w.closeObject()
	return w.String()
}

// writeElasticlunrIndex emits the elasticlunr `index` object: metadata, the
// per-field tries, the document store and the language marker.
func writeElasticlunrIndex(w *jsonWriter, idx *Index) {
	w.openObject()

	w.key("fields")
	w.openArray()
	for i, f := range idx.fields {
		if i > 0 {
			w.comma()
		}
		w.str(f)
	}
	w.closeArray()
	w.comma()

	w.key("pipeline")
	w.raw(`["trimmer","stopWordFilter","stemmer"]`)
	w.comma()
	w.key("ref")
	w.str("id")
	w.comma()
	w.key("version")
	w.str("0.9.5")
	w.comma()

	w.key("index")
	w.openObject()
	// The per-field map is a BTreeMap: body, breadcrumbs, title.
	for i, field := range sortedKeys(idx.trees) {
		if i > 0 {
			w.comma()
		}
		w.key(field)
		w.openObject()
		w.key("root")
		writeIndexItem(w, idx.trees[field])
		w.closeObject()
	}
	w.closeObject()
	w.comma()

	w.key("documentStore")
	w.openObject()
	w.key("save")
	w.bool(true)
	w.comma()
	w.key("docs")
	w.openObject()
	docs := append([]storedDoc(nil), idx.docs...)
	sort.Slice(docs, func(i, j int) bool { return docs[i].ref < docs[j].ref })
	for i, doc := range docs {
		if i > 0 {
			w.comma()
		}
		w.key(doc.ref)
		w.openObject()
		for j, name := range sortedKeys(doc.fields) {
			if j > 0 {
				w.comma()
			}
			w.key(name)
			w.str(doc.fields[name])
		}
		w.closeObject()
	}
	w.closeObject()
	w.comma()
	w.key("docInfo")
	w.openObject()
	for i, doc := range docs {
		if i > 0 {
			w.comma()
		}
		w.key(doc.ref)
		w.openObject()
		for j, name := range sortedKeys(doc.counts) {
			if j > 0 {
				w.comma()
			}
			w.key(name)
			w.int(doc.counts[name])
		}
		w.closeObject()
	}
	w.closeObject()
	w.comma()
	w.key("length")
	w.int(len(idx.docs))
	w.closeObject()
	w.comma()

	w.key("lang")
	w.str("English")
	w.closeObject()
}

// writeIndexItem emits one trie node: docs, df, then the children in key order.
func writeIndexItem(w *jsonWriter, item *indexItem) {
	w.openObject()
	w.key("docs")
	w.openObject()
	for i, ref := range sortedKeys(item.docs) {
		if i > 0 {
			w.comma()
		}
		w.key(ref)
		w.openObject()
		w.key("tf")
		w.float(item.docs[ref])
		w.closeObject()
	}
	w.closeObject()
	w.comma()
	w.key("df")
	w.int(item.df)
	for _, ch := range sortedKeys(item.children) {
		w.comma()
		w.key(ch)
		writeIndexItem(w, item.children[ch])
	}
	w.closeObject()
}

// sortedKeys returns a map's keys in the order a Rust BTreeMap would iterate
// them, which for string keys is byte order.
func sortedKeys[V any](m map[string]V) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}
