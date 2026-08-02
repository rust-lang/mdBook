package search

import (
	"os"
	"testing"
)

// basicFixtureDocs are the four documents the Rust renderer indexed for
// mdbook-go/fixtures/basic. They were read back out of the golden
// searchindex.js documentStore, so the test compares only the index builder.
var basicFixtureDocs = []Doc{
	{
		URL:         "intro.html#introduction",
		Title:       "Introduction",
		Body:        "Welcome to the basic fixture. This is used to verify that mdbook-go can\nload a book and render chapters. Item one Item two",
		Breadcrumbs: "Introduction \u00bb Introduction",
	},
	{
		URL:         "chapter_1.html#chapter-1",
		Title:       "Chapter 1",
		Body:        "First chapter. fn main() { println!(\"hello\");\n} a b 1 2",
		Breadcrumbs: "Chapter 1 \u00bb Chapter 1",
	},
	{
		URL:         "section_1_1.html#section-11",
		Title:       "Section 1.1",
		Body:        "Nested chapter.",
		Breadcrumbs: "Chapter 1 \u00bb Section 1.1 \u00bb Section 1.1",
	},
	{
		URL:         "chapter_2.html#chapter-2",
		Title:       "Chapter 2",
		Body:        "Second top-level chapter with a link to chapter 1.",
		Breadcrumbs: "Chapter 2 \u00bb Chapter 2",
	},
}

// defaultOptions mirrors the Rust defaults for [output.html.search].
func defaultOptions() Options {
	return Options{
		LimitResults:    30,
		TeaserWordCount: 30,
		BoostTitle:      2,
		BoostParagraph:  1,
		BoostHierarchy:  1,
		UseBooleanAnd:   false,
		Expand:          true,
	}
}

func TestIndexJSONMatchesRustGolden(t *testing.T) {
	want, err := os.ReadFile("testdata/searchindex.golden.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := IndexJSON(basicFixtureDocs, defaultOptions())
	if err != nil {
		t.Fatal(err)
	}
	if got != string(want) {
		i := 0
		for i < len(got) && i < len(want) && got[i] == want[i] {
			i++
		}
		lo := max(0, i-60)
		t.Fatalf("index differs at byte %d (got %d bytes, want %d)\n got: %q\nwant: %q",
			i, len(got), len(want), got[lo:min(len(got), i+120)], want[lo:min(len(want), i+120)])
	}
}
