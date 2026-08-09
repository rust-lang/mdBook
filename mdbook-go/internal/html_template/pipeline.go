package html_template

import "strings"

// englishStopWords is the elasticlunr-rs English stop-word list, copied
// verbatim from lang/en.rs. The first entry is intentionally "" because that
// is how the Rust list is constructed; it has no behavioural effect since
// no token is empty at the point the filter runs.
var englishStopWords = []string{
	"", "a", "able", "about", "across", "after", "all", "almost", "also", "am", "among", "an",
	"and", "any", "are", "as", "at", "be", "because", "been", "but", "by", "can", "cannot",
	"could", "dear", "did", "do", "does", "either", "else", "ever", "every", "for", "from", "get",
	"got", "had", "has", "have", "he", "her", "hers", "him", "his", "how", "however", "i", "if",
	"in", "into", "is", "it", "its", "just", "least", "let", "like", "likely", "may", "me",
	"might", "most", "must", "my", "neither", "no", "nor", "not", "of", "off", "often", "on",
	"only", "or", "other", "our", "own", "rather", "said", "say", "says", "she", "should", "since",
	"so", "some", "than", "that", "the", "their", "them", "then", "there", "these", "they", "this",
	"tis", "to", "too", "twas", "us", "wants", "was", "we", "were", "what", "when", "where",
	"which", "while", "who", "whom", "why", "will", "with", "would", "yet", "you", "your",
}

// englishStopWordsSet is the lookup table used by the stop-word filter. Built
// lazily on first use so startup stays cheap.
var englishStopWordsSet = func() map[string]struct{} {
	m := make(map[string]struct{}, len(englishStopWords))
	for _, w := range englishStopWords {
		m[w] = struct{}{}
	}
	return m
}()

// pipelineStep is a single stage of the elasticlunr-rs English pipeline:
// trimmer -> stopWordFilter -> stemmer. Each step mirrors the corresponding
// Rust closure; if it returns false the token is dropped.
type pipelineStep func(token string) (string, bool)

// runEnglishPipeline runs the full English pipeline over tokens, matching
// elasticlunr-rs's Pipeline::run.
func runEnglishPipeline(tokens []string) []string {
	out := make([]string, 0, len(tokens))
	for _, tok := range tokens {
		t := tok
		ok := true
		for _, step := range englishSteps {
			if !ok {
				break
			}
			t, ok = step(t)
		}
		if ok {
			out = append(out, t)
		}
	}
	return out
}

// englishSteps is the ordered pipeline. The order is fixed by Rust and is
// load-bearing for elasticlunr.js compatibility (search.rs uses the same
// names in the emitted JSON).
var englishSteps = []pipelineStep{englishTrimmer, englishStopWordFilter, englishStemmer}

// englishTrimmer strips leading and trailing characters that are neither
// alphanumeric (is_digit(36)) nor underscore. This matches elasticlunr-rs
// lang/en.rs trimmer exactly.
func englishTrimmer(token string) (string, bool) {
	// strings.TrimFunc with the same predicate.
	r := strings.TrimFunc(token, func(r rune) bool {
		if r == '_' {
			return false
		}
		// is_digit(36) -> is alphanumeric (letter or digit). The Rust
		// definition: "given a radix <= 36, an ascii char is a digit if
		// ascii digit < radix", and for radix=36 this includes 0-9 and a-z.
		// For non-ASCII chars it's never a digit.
		return !isASCIIAlphanumeric(r)
	})
	return r, r != ""
}

// englishStopWordFilter drops tokens that appear in the English stop-word
// list. The empty string in the list is harmless (no token ever equals it).
func englishStopWordFilter(token string) (string, bool) {
	if _, ok := englishStopWordsSet[token]; ok {
		return "", false
	}
	return token, true
}

// englishStemmer applies the Porter stemmer (lang/en.rs PorterStemmer).
func englishStemmer(token string) (string, bool) {
	if len(token) <= 2 {
		return token, true
	}
	s, err := porterStem(token)
	if err != nil || s == "" {
		return "", false
	}
	return s, true
}

// isASCIIAlphanumeric reports whether r is an ASCII letter or digit. The
// Rust trimmer uses char::is_digit(36), which is ASCII-only for radix<=36.
func isASCIIAlphanumeric(r rune) bool {
	return (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9')
}
