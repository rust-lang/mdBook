package summary

import (
	"strings"
	"testing"
)

// TestParseDropsBareLinkInsideList verifies that a bare link indented
// deeper than the surrounding list item is discarded rather than
// misclassified as a suffix chapter. This matches Rust mdBook's
// behaviour, which uses pulldown-cmark events: a `[name](path)` line
// inside a list-item paragraph is treated as plain text and any link
// inside it is dropped.
//
// The regression fixture is `tests/testsuite/toc/basic_toc`, whose
// SUMMARY.md contains:
//
//     - [Deep Nest 3](deep/a/b/index.md)
//                 [Deep Nest 4](deep/a/b/c/index.md)
//
// Without the fix, our parser would append Deep Nest 4 as a suffix
// chapter, then fail the build because deep/a/b/c/index.md does not
// exist (Rust silently drops the entry and succeeds).
func TestParseDropsBareLinkInsideList(t *testing.T) {
	src := `# Summary

[Prefix 1](prefix1.md)

- [With Readme](README.md)
    - [Nested Index](nested/index.md)
- [Draft]()

---

# Deep Nest

- [Deep Nest 1](deep/index.md)
    - [Deep Nest 2](deep/a/index.md)
        - [Deep Nest 3](deep/a/b/index.md)
            [Deep Nest 4](deep/a/b/c/index.md)

---

[Suffix 1](suffix1.md)
`
	sum, err := Parse(src)
	if err != nil {
		t.Fatalf("Parse: %v", err)
	}

	// Suffix chapters should be exactly [Suffix 1].
	if got, want := len(sum.SuffixChapters), 1; got != want {
		t.Errorf("SuffixChapters count = %d, want %d", got, want)
	}
	if len(sum.SuffixChapters) > 0 && sum.SuffixChapters[0].Link != nil {
		if got, want := sum.SuffixChapters[0].Link.Name, "Suffix 1"; got != want {
			t.Errorf("SuffixChapters[0].Name = %q, want %q", got, want)
		}
	}

	// Numbered chapters must contain Deep Nest 1/2/3 but NOT Deep Nest 4.
	var names []string
	for _, it := range sum.NumberedChapters {
		if it.Link != nil {
			names = append(names, it.Link.Name)
			for _, sub := range it.Link.NestedItems {
				if sub.Link != nil {
					names = append(names, sub.Link.Name)
					for _, sub2 := range sub.Link.NestedItems {
						if sub2.Link != nil {
							names = append(names, sub2.Link.Name)
							for _, sub3 := range sub2.Link.NestedItems {
								if sub3.Link != nil {
									names = append(names, sub3.Link.Name)
								}
							}
						}
					}
				}
			}
		}
	}
	joined := strings.Join(names, ",")
	for _, want := range []string{"Deep Nest 1", "Deep Nest 2", "Deep Nest 3"} {
		if !strings.Contains(joined, want) {
			t.Errorf("expected %q in numbered chapters, got %q", want, joined)
		}
	}
	if strings.Contains(joined, "Deep Nest 4") {
		t.Errorf("Deep Nest 4 must be discarded (malformed entry), got %q", joined)
	}
}

// TestParseBareLinkAtListLevel ensures that a bare link at the same
// indent as the topmost list item is NOT swallowed by the "drop" rule.
// We don't have a Rust fixture exercising this; the test exists to
// guard against an over-broad fix that drops legitimate suffix entries
// appearing at non-zero indent.
func TestParseBareLinkAtListLevel(t *testing.T) {
	src := `# Summary

- [A](a.md)

[B](b.md)
`
	sum, err := Parse(src)
	if err != nil {
		t.Fatalf("Parse: %v", err)
	}
	// [B] at indent 0 with an empty stack should still become a suffix
	// chapter — same-level bare link after a numbered list is fine.
	if got := len(sum.SuffixChapters); got != 1 {
		t.Fatalf("SuffixChapters = %d, want 1", got)
	}
	if sum.SuffixChapters[0].Link == nil || sum.SuffixChapters[0].Link.Name != "B" {
		t.Errorf("SuffixChapters[0] = %+v, want Link{Name:\"B\"}", sum.SuffixChapters[0])
	}
}