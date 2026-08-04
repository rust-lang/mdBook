// Package summary parses SUMMARY.md into a list of top-level SummaryItems.
// It deliberately uses a simple line/state machine rather than a full
// markdown parser, which keeps the error messages close to the source. This
// mirrors the behaviour of mdbook-summary while being easier to reason about
// in Go.
package summary

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
)

// Link describes one chapter link in the summary.
type Link struct {
	Name        string        // visible name
	Location    string        // source path, e.g. "chapter_1.md"
	Number      []int         // 1-indexed chapter numbers; empty for drafts
	NestedItems []SummaryItem // children
	Level       int           // indent in spaces
}

// Separator represents a horizontal rule between groups.
type Separator struct {
	Level int
}

// PartTitle is a heading before a numbered group.
type PartTitle struct {
	Name  string
	Level int
}

// SummaryItem is the tagged union used in the tree.
type SummaryItem struct {
	Link      *Link
	Separator *Separator
	PartTitle *PartTitle
}

// Summary is the parsed form of SUMMARY.md.
type Summary struct {
	Title            string
	PrefixChapters   []SummaryItem
	NumberedChapters []SummaryItem
	SuffixChapters   []SummaryItem
	Source           string // raw text for diagnostics
}

// ParseFile reads and parses a SUMMARY.md file.
func ParseFile(path string) (*Summary, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read %s: %w", path, err)
	}
	return Parse(string(data))
}

var (
	// listLinkRe matches a list item holding a link, e.g. `  - [Name](path.md)`.
	listLinkRe = regexp.MustCompile(`^(\s*)[-*+]\s+\[(.*?)\]\((.*?)\)\s*$`)
	// bareLinkRe matches a link that is not a list item, which is how prefix
	// and suffix chapters are written.
	bareLinkRe = regexp.MustCompile(`^\s*\[(.*?)\]\((.*?)\)\s*$`)
	sepRe      = regexp.MustCompile(`^\s*-{3,}\s*$`)
	partRe     = regexp.MustCompile(`^\s*#\s+(.+?)\s*$`)
)

// Parse implements the SUMMARY.md grammar. The accepted grammar matches the
// documented mdBook subset:
//
//	# Title                        optional, one occurrence
//	[Name](path.md)                prefix chapter (before the numbered list)
//	# Part Title                   part title, opens the numbered section
//	- [Name](path.md)              numbered chapter, nestable by indentation
//	- [Name]()                     draft chapter, still consumes a number
//	---                            separator
//	[Name](path.md)                suffix chapter (after the numbered list)
//
// Whether a link is numbered is decided by the list marker, not by position:
// a bare link before the first list item is a prefix chapter, and one after is
// a suffix chapter.
func Parse(source string) (*Summary, error) {
	sum := &Summary{Source: source}
	scanner := bufio.NewScanner(strings.NewReader(source))
	scanner.Buffer(make([]byte, 0, 64*1024), 1024*1024)

	var (
		inNumbered bool
		seenTitle  bool
		lineNo     int
		// stack holds the open numbered chapters, innermost last, so a link can
		// be attached to the closest parent with a smaller indent.
		stack []*Link
	)

	for scanner.Scan() {
		lineNo++
		raw := scanner.Text()
		trimmed := strings.TrimSpace(raw)
		if trimmed == "" {
			continue
		}

		if !seenTitle && !inNumbered && strings.HasPrefix(trimmed, "# ") &&
			len(sum.PrefixChapters) == 0 {
			sum.Title = strings.TrimSpace(strings.TrimPrefix(trimmed, "# "))
			seenTitle = true
			continue
		}

		if sepRe.MatchString(raw) {
			stack = nil
			item := SummaryItem{Separator: &Separator{Level: leadingSpaces(raw)}}
			if inNumbered {
				sum.NumberedChapters = append(sum.NumberedChapters, item)
			} else {
				sum.PrefixChapters = append(sum.PrefixChapters, item)
			}
			continue
		}

		if m := partRe.FindStringSubmatch(raw); m != nil {
			inNumbered = true
			stack = nil
			sum.NumberedChapters = append(sum.NumberedChapters,
				SummaryItem{PartTitle: &PartTitle{Name: m[1], Level: leadingSpaces(raw)}})
			continue
		}

		if m := listLinkRe.FindStringSubmatch(raw); m != nil {
			inNumbered = true
			level := len(m[1])
			// Strip leading "./" so chapter Path stays "comment-in-list.md" rather
			// than "./comment-in-list.md" — matches Rust's SUMMARY parser.
			location := strings.TrimPrefix(m[3], "./")
			link := &Link{Name: m[2], Location: location, Level: level}

			// Close every open chapter indented at least as far as this one,
			// then attach to whatever remains open.
			for len(stack) > 0 && stack[len(stack)-1].Level >= level {
				stack = stack[:len(stack)-1]
			}
			if len(stack) == 0 {
				sum.NumberedChapters = append(sum.NumberedChapters, SummaryItem{Link: link})
			} else {
				parent := stack[len(stack)-1]
				parent.NestedItems = append(parent.NestedItems, SummaryItem{Link: link})
			}
			stack = append(stack, link)
			continue
		}

		if m := bareLinkRe.FindStringSubmatch(raw); m != nil {
			level := leadingSpaces(raw)
			// A bare link indented deeper than the innermost open list
			// item is a continuation paragraph of that list item, not a
			// new chapter. Rust's mdbook-summary uses pulldown-cmark
			// events; pulldown-cmark parses `[X](y)` inside a list item
			// as plain text inside that item's paragraph and the link is
			// discarded. Discarding here matches the
			// `tests/testsuite/toc/basic_toc` SUMMARY.md, which has a
			// malformed deeper-indented entry.
			if len(stack) > 0 && level > stack[len(stack)-1].Level {
				continue
			}
			stack = nil
			item := SummaryItem{Link: &Link{Name: m[1], Location: strings.TrimPrefix(m[2], "./"), Level: level}}
			if inNumbered {
				sum.SuffixChapters = append(sum.SuffixChapters, item)
			} else {
				sum.PrefixChapters = append(sum.PrefixChapters, item)
			}
			continue
		}

		return nil, fmt.Errorf("line %d: unrecognised SUMMARY.md syntax: %q", lineNo, raw)
	}
	if err := scanner.Err(); err != nil {
		return nil, err
	}
	return sum, nil
}

func leadingSpaces(s string) int {
	return len(s) - len(strings.TrimLeft(s, " \t"))
}

// NumberStrings returns the rendered numbers, e.g. ["1", "1.2"].
func NumberStrings(n []int) []string {
	out := make([]string, 0, len(n))
	for _, v := range n {
		out = append(out, strconv.Itoa(v))
	}
	return out
}

// ResolveLocation joins a SUMMARY-relative location with the book root.
func ResolveLocation(root, location string) string {
	if location == "" {
		return ""
	}
	if filepath.IsAbs(location) {
		return location
	}
	return filepath.Join(root, location)
}
