package utils

import (
	"fmt"
	"strings"
	"unicode"
)

// IDFromContent slugifies heading text the same way id_from_content does in
// crates/mdbook-html/src/utils.rs: trim, lowercase, keep alphanumerics plus
// `_` and `-`, turn whitespace into `-`, drop everything else.
func IDFromContent(content string) string {
	var b strings.Builder
	b.Grow(len(content))
	for _, ch := range strings.ToLower(strings.TrimSpace(content)) {
		switch {
		case unicode.IsLetter(ch) || unicode.IsDigit(ch) || ch == '_' || ch == '-':
			b.WriteRune(ch)
		case unicode.IsSpace(ch):
			b.WriteRune('-')
		}
	}
	return b.String()
}

// IDSet tracks the identifiers handed out so far so duplicates can be
// disambiguated. The zero value is not usable; call NewIDSet.
type IDSet struct {
	used map[string]struct{}
}

// NewIDSet returns an empty identifier set.
func NewIDSet() *IDSet {
	return &IDSet{used: make(map[string]struct{})}
}

// Unique returns id if it has not been handed out yet, otherwise it appends
// `-1`, `-2`, ... until an unused candidate is found. This mirrors unique_id in
// crates/mdbook-html/src/utils.rs.
func (s *IDSet) Unique(id string) string {
	if _, ok := s.used[id]; !ok {
		s.used[id] = struct{}{}
		return id
	}
	for counter := 1; ; counter++ {
		candidate := fmt.Sprintf("%s-%d", id, counter)
		if _, ok := s.used[candidate]; !ok {
			s.used[candidate] = struct{}{}
			return candidate
		}
	}
}

// Has reports whether the identifier was already handed out.
func (s *IDSet) Has(id string) bool {
	_, ok := s.used[id]
	return ok
}

// Add records an identifier as used without returning a candidate.
func (s *IDSet) Add(id string) {
	s.used[id] = struct{}{}
}
