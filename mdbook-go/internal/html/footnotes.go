package html

import (
	"sort"
	"strconv"

	extast "github.com/yuin/goldmark/extension/ast"
)

// footnoteReference emits `<sup class="footnote-reference" id="fr-NAME-N"><a
// href="#footnote-NAME">N</a></sup>` and records the usage so the definition
// can link back. Ported from footnote_reference in
// crates/mdbook-html/src/html/tree.rs.
func (b *builder) footnoteReference(n *extast.FootnoteLink) error {
	name := b.footnoteName(n.Index)
	info, seen := b.footnoteNumbers[name]
	if !seen {
		info = &footnoteInfo{number: len(b.footnoteNumbers) + 1}
		b.footnoteNumbers[name] = info
		b.footnoteOrder = append(b.footnoteOrder, name)
	}
	info.uses++

	// Two references that end up side by side are separated by a space so the
	// superscripts do not run together.
	if last := b.cur.LastChild(); last != nil && last.Kind == KindElement && last.El.Name == "sup" {
		if class, ok := last.El.Attr("class"); ok && class == "footnote-reference" {
			b.append(NewText(" "))
		}
	}

	sup := NewElement("sup")
	sup.El.SetAttr("class", "footnote-reference")
	sup.El.SetAttr("id", "fr-"+name+"-"+strconv.Itoa(info.uses))
	a := NewElement("a")
	a.El.SetAttr("href", "#footnote-"+name)
	a.Append(NewText(strconv.Itoa(info.number)))
	sup.Append(a)
	b.append(sup)
	return nil
}

// footnoteDefinition collects the definition body into a detached <li>. The
// definitions are appended to the document by collectFootnoteDefs.
func (b *builder) footnoteDefinition(n *extast.Footnote) error {
	name := string(n.Ref)
	if _, dup := b.footnoteDefs[name]; dup {
		return nil
	}
	li := NewElement("li")
	li.El.SetAttr("id", "footnote-"+name)

	saved := b.cur
	b.cur = li
	err := b.walk(n)
	b.cur = saved
	if err != nil {
		return err
	}
	b.footnoteDefs[name] = li
	return nil
}

// footnoteName resolves a goldmark footnote index back to its label.
func (b *builder) footnoteName(index int) string {
	if name, ok := b.footnoteIndexNames[index]; ok {
		return name
	}
	return strconv.Itoa(index)
}

// collectFootnoteDefs appends `<hr>` plus the `<ol class="footnote-definition">`
// list, in first-reference order, with `↩` back-links. Definitions that were
// never referenced are dropped. Ported from collect_footnote_defs.
func (b *builder) collectFootnoteDefs() {
	if len(b.footnoteDefs) == 0 {
		return
	}
	names := make([]string, 0, len(b.footnoteDefs))
	for name := range b.footnoteDefs {
		if _, referenced := b.footnoteNumbers[name]; referenced {
			names = append(names, name)
		}
	}
	if len(names) == 0 {
		return
	}
	sort.Slice(names, func(i, j int) bool {
		return b.footnoteNumbers[names[i]].number < b.footnoteNumbers[names[j]].number
	})

	b.root.Append(NewElement("hr"))
	ol := NewElement("ol")
	ol.El.SetAttr("class", "footnote-definition")
	b.root.Append(ol)

	for _, name := range names {
		def := b.footnoteDefs[name]
		for usage := 1; usage <= b.footnoteNumbers[name].uses; usage++ {
			nth := ""
			if usage > 1 {
				nth = strconv.Itoa(usage)
			}
			backlink := NewElement("a")
			backlink.El.SetAttr("href", "#fr-"+name+"-"+strconv.Itoa(usage))
			backlink.Append(NewText("↩" + nth))

			// The back-link goes inside the last paragraph when there is one,
			// so it sits on the same line as the definition text.
			target := def
			if last := def.LastChild(); last != nil && last.Kind == KindElement && last.El.Name == "p" {
				target = last
			}
			target.Append(NewText(" "))
			target.Append(backlink)
		}
		ol.Append(def)
	}
}
