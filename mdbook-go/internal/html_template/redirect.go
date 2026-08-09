package html_template

import (
	"encoding/json"
	"fmt"
	"html/template"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"

	"mdbook-go/pkg/fs"
)

// emitRedirects writes one small HTML page per configured redirect.
func emitRedirects(destination string, registry *Registry, redirects map[string]string) error {
	if len(redirects) == 0 {
		return nil
	}
	combined := combineFragmentRedirects(redirects)
	for _, original := range sortedMapKeys(combined) {
		entry := combined[original]
		target := filepath.Join(destination, filepath.FromSlash(strings.TrimPrefix(original, "/")))
		if _, err := os.Stat(target); err == nil {
			continue
		}
		if entry.destination == "" {
			return fmt.Errorf("redirect entry for %q has no destination", original)
		}
		fragmentJSON, err := json.Marshal(entry.fragments)
		if err != nil {
			return err
		}
		// The redirect template (templates/redirect.html) reads {{.URL}} and
		// {{.FragmentMap}}; the keys must match or html/template renders the
		// missing map entries as empty (and "null" in JS context). The two
		// //-comment lines and the in-script URL are passed as typed JS too:
		// html/template strips bare JS line comments in <script> context and
		// JS-escapes plain strings there (e.g. "/" becomes "\/").
		page, err := registry.Render("redirect", map[string]any{
			"URL":         template.URL(entry.destination),
			"URLJS":       template.JS(strconv.Quote(entry.destination)),
			"FragmentMap": template.JS(string(fragmentJSON)),
			"Comment1":    template.JS("// This handles redirects that involve fragments."),
			"Comment2":    template.JS("// else redirect handled by http-equiv"),
		})
		if err != nil {
			return err
		}
		if err := fs.WriteFile(target, []byte(page)); err != nil {
			return err
		}
	}
	return nil
}

type redirectEntry struct {
	destination string
	fragments   map[string]string
}

func combineFragmentRedirects(redirects map[string]string) map[string]*redirectEntry {
	combined := make(map[string]*redirectEntry)
	get := func(key string) *redirectEntry {
		if entry, ok := combined[key]; ok {
			return entry
		}
		entry := &redirectEntry{fragments: map[string]string{}}
		combined[key] = entry
		return entry
	}
	for source, target := range redirects {
		page, fragment, hasFragment := strings.Cut(source, "#")
		if hasFragment {
			get(page).fragments["#"+fragment] = target
			continue
		}
		get(page).destination = target
	}
	return combined
}

func sortedMapKeys[V any](m map[string]V) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}
