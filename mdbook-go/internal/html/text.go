package html

import (
	"strconv"
	"strings"
	"unicode/utf8"

	"github.com/yuin/goldmark/util"
)

// unescapeText resolves the backslash escapes and character references inside a
// Markdown text run. goldmark leaves both in the source segment and only
// resolves them while rendering, but this port builds a node tree first and
// escapes on serialization, so the resolution has to happen here.
//
// The algorithm follows goldmark's defaultWriter.Write
// (renderer/html/html.go) minus its HTML escaping.
func unescapeText(source []byte) string {
	var out strings.Builder
	out.Grow(len(source))

	escaped := false
	limit := len(source)
	n := 0
	for i := 0; i < limit; i++ {
		c := source[i]
		if escaped && util.IsPunct(c) {
			out.Write(source[n : i-1])
			n = i
			escaped = false
			continue
		}
		if c == 0 {
			out.Write(source[n:i])
			out.WriteRune(utf8.RuneError)
			n = i + 1
			escaped = false
			continue
		}
		if c == '&' {
			pos := i
			next := i + 1
			if next < limit && source[next] == '#' {
				if nnext := next + 1; nnext < limit {
					switch nc := source[nnext]; {
					case nc == 'x' || nc == 'X':
						start := nnext + 1
						end, ok := util.ReadWhile(source, [2]int{start, limit}, util.IsHexDecimal)
						if ok && end < limit && source[end] == ';' && end-start < 7 {
							v, _ := strconv.ParseUint(string(source[start:end]), 16, 32)
							out.Write(source[n:pos])
							out.WriteRune(sanitizeRune(rune(v)))
							i, n = end, end+1
							continue
						}
					case nc >= '0' && nc <= '9':
						start := nnext
						end, ok := util.ReadWhile(source, [2]int{start, limit}, util.IsNumeric)
						if ok && end < limit && end-start < 8 && source[end] == ';' {
							v, _ := strconv.ParseUint(string(source[start:end]), 10, 32)
							out.Write(source[n:pos])
							out.WriteRune(sanitizeRune(rune(v)))
							i, n = end, end+1
							continue
						}
					}
				}
			} else {
				start := next
				end, ok := util.ReadWhile(source, [2]int{start, limit}, util.IsAlphaNumeric)
				if ok && end < limit && source[end] == ';' {
					if entity, found := util.LookUpHTML5EntityByName(string(source[start:end])); found {
						out.Write(source[n:pos])
						out.Write(entity.Characters)
						i, n = end, end+1
						continue
					}
				}
			}
			i = next - 1
		}
		if c == '\\' {
			escaped = true
			continue
		}
		escaped = false
	}
	out.Write(source[n:])
	return out.String()
}

// sanitizeRune maps the code points that must not appear in output to the
// Unicode replacement character, as CommonMark requires.
func sanitizeRune(r rune) rune {
	if r == 0 || !utf8.ValidRune(r) {
		return utf8.RuneError
	}
	return r
}
