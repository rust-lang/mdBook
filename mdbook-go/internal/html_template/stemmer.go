package html_template

import (
	"strings"
)

// porterStem applies the Porter stemmer as ported from elasticlunr-rs
// lang/en.rs. The algorithm operates on a byte buffer that is mutated in
// place; on success the stemmed prefix of the buffer is returned.
//
// For UTF-8 tokens that are not valid ASCII, the stemmer still operates on
// raw bytes (matching the Rust implementation), but every rule is guarded so
// that `k` never lands mid-character, leaving valid UTF-8 in b[0..k].
func porterStem(word string) (string, error) {
	if len(word) <= 2 {
		return strings.ToLower(word), nil
	}
	ps, err := newPorterStemmer(word)
	if err != nil {
		return "", err
	}
	if err := ps.run(); err != nil {
		return "", err
	}
	return ps.get(), nil
}

// porterStemmer mirrors elasticlunr-rs's PorterStemmer struct. `b` is the
// mutable byte buffer, `k` is the current length, `j` is set as a side
// effect by ends() to record where a suffix match started.
type porterStemmer struct {
	b []byte
	k int
	j int
}

func newPorterStemmer(word string) (*porterStemmer, error) {
	// Rust uses to_ascii_lowercase(). For UTF-8 input the lowercase of
	// non-ASCII runes is unchanged by to_ascii_lowercase, so we leave them
	// alone. Words without any ASCII letter (for example the section number
	// "1.1") simply match none of the rules and come back unchanged, which is
	// what elasticlunr-rs does.
	return &porterStemmer{b: []byte(word), k: len(word), j: 0}, nil
}

func isASCIILetter(c byte) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

func (p *porterStemmer) run() error {
	p.step1ab()
	p.step1c()
	p.step2()
	p.step3()
	p.step4()
	p.step5()
	return nil
}

func (p *porterStemmer) get() string {
	return string(p.b[:p.k])
}

// isConsonant reports whether b[i] is a consonant. Matches the Rust impl
// (y is consonant at index 0 and after a consonant, vowel otherwise).
func (p *porterStemmer) isConsonant(i int) bool {
	c := p.b[i]
	switch c {
	case 'a', 'e', 'i', 'o', 'u':
		return false
	case 'y':
		if i == 0 {
			return true
		}
		return !p.isConsonant(i - 1)
	default:
		return true
	}
}

// measure counts the (C)(VC)^m V? pattern over the stem b[0..j), which is the
// classic Porter `m()`. j is an exclusive end offset here, so C's `i > j`
// becomes `i >= j`.
func (p *porterStemmer) measure() int {
	j := p.j
	n := 0
	i := 0
	for {
		if i >= j {
			return n
		}
		if !p.isConsonant(i) {
			break
		}
		i++
	}
	i++
	for {
		for {
			if i >= j {
				return n
			}
			if p.isConsonant(i) {
				break
			}
			i++
		}
		i++
		n++
		for {
			if i >= j {
				return n
			}
			if !p.isConsonant(i) {
				break
			}
			i++
		}
		i++
	}
}

func (p *porterStemmer) hasVowel() bool {
	for i := 0; i < p.j; i++ {
		if !p.isConsonant(i) {
			return true
		}
	}
	return false
}

// doubleConsonant matches Rust's "i,(i-1) contain a double consonant" rule,
// restricted to ASCII so we never truncate multi-byte UTF-8 in the middle.
func (p *porterStemmer) doubleConsonant(i int) bool {
	if i < 1 {
		return false
	}
	c := p.b[i]
	if c != p.b[i-1] {
		return false
	}
	if c > 127 {
		return false
	}
	return p.isConsonant(i)
}

// cvc reports the consonant-vowel-consonant pattern, where the final
// consonant is not w, x or y (so we never restore a trailing e to those).
func (p *porterStemmer) cvc(i int) bool {
	if i < 2 {
		return false
	}
	if !p.isConsonant(i) || p.isConsonant(i-1) || !p.isConsonant(i-2) {
		return false
	}
	switch p.b[i] {
	case 'w', 'x', 'y':
		return false
	}
	return true
}

// ends reports whether b[0..k] ends with s, recording the prefix length in j.
func (p *porterStemmer) ends(s string) bool {
	if len(s) > p.k {
		return false
	}
	off := p.k - len(s)
	if string(p.b[off:p.k]) != s {
		return false
	}
	p.j = off
	return true
}

// setTo writes s into b[j..j+len(s)] and updates k = j + len(s). Bytes
// beyond the new k are left intact (and discarded by get()).
func (p *porterStemmer) setTo(s string) {
	copy(p.b[p.j:p.j+len(s)], s)
	p.k = p.j + len(s)
}

// r applies setTo(s) only if the current stem has measure > 0.
func (p *porterStemmer) r(s string) {
	if p.measure() > 0 {
		p.setTo(s)
	}
}

func (p *porterStemmer) step1ab() {
	if p.b[p.k-1] == 's' {
		switch {
		case p.ends("sses"):
			p.k -= 2
		case p.ends("ies"):
			p.setTo("i")
		case p.b[p.k-2] != 's':
			p.k--
		}
	}
	if p.ends("eed") {
		if p.measure() > 0 {
			p.k--
		}
	} else if (p.ends("ed") || p.ends("ing")) && p.hasVowel() {
		p.k = p.j
		switch {
		case p.ends("at"):
			p.setTo("ate")
		case p.ends("bl"):
			p.setTo("ble")
		case p.ends("iz"):
			p.setTo("ize")
		case p.doubleConsonant(p.k - 1):
			p.k--
			switch p.b[p.k-1] {
			case 'l', 's', 'z':
				p.k++
			}
		case p.measure() == 1 && p.cvc(p.k-1):
			p.setTo("e")
		}
	}
}

// step1c: turn terminal y to i unless the preceding char is a real vowel.
// Matches elasticlunr.js's regex rule exactly, so "leeshyy" -> "leeshyi"
// while "lay" stays "lay". The Rust impl checks b[k-2] against aeiou
// literally, which is what we do here.
func (p *porterStemmer) step1c() {
	if p.k <= 2 || !p.ends("y") {
		return
	}
	switch p.b[p.k-2] {
	case 'a', 'e', 'i', 'o', 'u':
		// preceding char is a vowel, leave alone
	default:
		p.b[p.k-1] = 'i'
	}
}

func (p *porterStemmer) step2() {
	if p.k < 2 {
		return
	}
	switch p.b[p.k-2] {
	case 'a':
		switch {
		case p.ends("ational"):
			p.r("ate")
			return
		case p.ends("tional"):
			p.r("tion")
		}
	case 'c':
		switch {
		case p.ends("enci"):
			p.r("ence")
			return
		case p.ends("anci"):
			p.r("ance")
		}
	case 'e':
		if p.ends("izer") {
			p.r("ize")
		}
	case 'l':
		switch {
		case p.ends("bli"):
			p.r("ble")
			return
		case p.ends("alli"):
			p.r("al")
			return
		case p.ends("entli"):
			p.r("ent")
			return
		case p.ends("eli"):
			p.r("e")
			return
		case p.ends("ousli"):
			p.r("ous")
		}
	case 'o':
		switch {
		case p.ends("ization"):
			p.r("ize")
			return
		case p.ends("ation"):
			p.r("ate")
			return
		case p.ends("ator"):
			p.r("ate")
		}
	case 's':
		switch {
		case p.ends("alism"):
			p.r("al")
			return
		case p.ends("iveness"):
			p.r("ive")
			return
		case p.ends("fulness"):
			p.r("ful")
			return
		case p.ends("ousness"):
			p.r("ous")
		}
	case 't':
		switch {
		case p.ends("aliti"):
			p.r("al")
			return
		case p.ends("iviti"):
			p.r("ive")
			return
		case p.ends("biliti"):
			p.r("ble")
		}
	case 'g':
		if p.ends("logi") {
			p.r("log")
		}
	}
}

func (p *porterStemmer) step3() {
	switch p.b[p.k-1] {
	case 'e':
		switch {
		case p.ends("icate"):
			p.r("ic")
			return
		case p.ends("ative"):
			p.r("")
			return
		case p.ends("alize"):
			p.r("al")
		}
	case 'i':
		if p.ends("iciti") {
			p.r("ic")
		}
	case 'l':
		switch {
		case p.ends("ical"):
			p.r("ic")
			return
		case p.ends("ful"):
			p.r("")
		}
	case 's':
		if p.ends("ness") {
			p.r("")
		}
	}
}

func (p *porterStemmer) step4() {
	if p.k < 2 {
		return
	}
	var matched bool
	switch p.b[p.k-2] {
	case 'a':
		matched = p.ends("al")
	case 'c':
		matched = p.ends("ance") || p.ends("ence")
	case 'e':
		matched = p.ends("er")
	case 'i':
		matched = p.ends("ic")
	case 'l':
		matched = p.ends("able") || p.ends("ible")
	case 'n':
		matched = p.ends("ant") || p.ends("ement") || p.ends("ment") || p.ends("ent")
	case 'o':
		matched = (p.ends("ion") && p.j > 0 && (p.b[p.j-1] == 's' || p.b[p.j-1] == 't')) || p.ends("ou")
	case 's':
		matched = p.ends("ism")
	case 't':
		matched = p.ends("ate") || p.ends("iti")
	case 'u':
		matched = p.ends("ous")
	case 'v':
		matched = p.ends("ive")
	case 'z':
		matched = p.ends("ize")
	}
	if matched && p.measure() > 1 {
		p.k = p.j
	}
}

func (p *porterStemmer) step5() {
	p.j = p.k
	if p.b[p.k-1] == 'e' {
		a := p.measure()
		if a > 1 || (a == 1 && !p.cvc(p.k-2)) {
			p.k--
		}
	}
	if p.b[p.k-1] == 'l' && p.doubleConsonant(p.k-1) && p.measure() > 1 {
		p.k--
	}
}
