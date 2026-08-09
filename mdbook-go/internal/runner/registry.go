package runner

import (
	"fmt"
	"sort"

	"mdbook-go/internal/model"
	"mdbook-go/internal/plugin"
)

// Built-in preprocessor names that ship with mdBook. Users can disable them
// by setting `build.use-default-preprocessors = false` in doclens.yaml.
var defaultPreprocessors = []string{"links", "index"}

// PreprocessorConfigEntry captures the per-name knobs the user can set in
// doclens.yaml under the preprocessor.<name> section.
type PreprocessorConfigEntry struct {
	Command   string
	Before    []string
	After     []string
	Optional  bool
	Renderers []string
}

// parsePreprocessorConfig turns the dynamic preprocessor map into typed
// entries. Unknown keys inside each table are ignored — they are forwarded
// verbatim to the preprocessor via the wire context.
func parsePreprocessorConfig(cfg *model.Config) map[string]PreprocessorConfigEntry {
	out := map[string]PreprocessorConfigEntry{}
	for name, raw := range cfg.Preprocessor {
		entry := PreprocessorConfigEntry{}
		if tbl, ok := raw.(map[string]any); ok {
			if v, ok := tbl["command"].(string); ok {
				entry.Command = v
			}
			entry.Before = stringList(tbl["before"])
			entry.After = stringList(tbl["after"])
			if v, ok := tbl["optional"].(bool); ok {
				entry.Optional = v
			}
			entry.Renderers = stringList(tbl["renderers"])
		}
		out[name] = entry
	}
	return out
}

// stringList coerces a slice of strings or a single string into a []string.
func stringList(v any) []string {
	switch t := v.(type) {
	case []any:
		out := make([]string, 0, len(t))
		for _, x := range t {
			if s, ok := x.(string); ok {
				out = append(out, s)
			}
		}
		return out
	case []string:
		return t
	case string:
		return []string{t}
	}
	return nil
}

// BuildPreprocessors resolves the configured preprocessors for a book,
// applying the topological ordering rules from
// crates/mdbook-driver/src/mdbook.rs::determine_preprocessors.
//
// The returned slice is in execution order. The default preprocessors are
// included only if cfg.Build.UseDefaultPreprocessors is true.
func BuildPreprocessors(cfg *model.Config, root string) ([]plugin.Preprocessor, error) {
	table := parsePreprocessorConfig(cfg)

	// Collect the names that should participate in the ordering.
	var names []string
	if cfg.Build.UseDefaultPreprocessors {
		for _, n := range defaultPreprocessors {
			if !contains(names, n) {
				names = append(names, n)
			}
		}
	}
	for n := range table {
		if !contains(names, n) {
			names = append(names, n)
		}
	}

	// Kahn-style topological sort. Edges: a "before X" on Y means Y depends
	// on X (X must run first). Edges: an "after X" on Y means Y depends on X.
	indeg := map[string]int{}
	adj := map[string][]string{} // dependent -> predecessors
	exists := func(n string) bool {
		if cfg.Build.UseDefaultPreprocessors && contains(defaultPreprocessors, n) {
			return true
		}
		_, ok := table[n]
		return ok
	}

	addEdge := func(from, to string) {
		if !exists(from) {
			return
		}
		adj[from] = append(adj[from], to)
		indeg[to]++
		if _, ok := indeg[from]; !ok {
			indeg[from] = 0
		}
	}

	for _, name := range names {
		entry := table[name]
		for _, before := range entry.Before {
			// `name` runs before `before` — edge from `name` to `before`.
			addEdge(name, before)
		}
		for _, after := range entry.After {
			// `name` runs after `after` — edge from `after` to `name`.
			addEdge(after, name)
		}
		if _, ok := indeg[name]; !ok {
			indeg[name] = 0
		}
	}

	var ordered []plugin.Preprocessor
	for {
		// Find every name whose indegree is 0.
		var ready []string
		for _, n := range names {
			if d, ok := indeg[n]; ok && d == 0 {
				ready = append(ready, n)
			}
		}
		if len(ready) == 0 {
			break
		}
		sort.Strings(ready) // stable tie-break matching Rust
		for _, n := range ready {
			delete(indeg, n)
			for _, succ := range adj[n] {
				indeg[succ]--
			}
			ordered = append(ordered, buildOne(n, table, root))
		}
	}
	if len(indeg) > 0 {
		return nil, fmt.Errorf("cyclic dependency detected in preprocessors: %v", indeg)
	}
	return ordered, nil
}

func buildOne(name string, table map[string]PreprocessorConfigEntry, root string) plugin.Preprocessor {
	switch name {
	case "links":
		return LinkPreprocessor{}
	case "index":
		return IndexPreprocessor{}
	}
	entry := table[name]
	cmd := entry.Command
	if cmd == "" {
		cmd = "mdbook-" + name
	}
	return NewCmdPreprocessor(name, cmd, root, entry.Optional)
}

// BuildRenderers resolves the configured renderers. The built-in HTML
// renderer is provided by internal/html_template; this helper only handles
// the custom command renderers the user registers.
func BuildRenderers(cfg *model.Config) (map[string]plugin.Renderer, error) {
	out := map[string]plugin.Renderer{}
	for name, raw := range cfg.Output {
		if name == "html" || name == "markdown" {
			continue // handled by the runner
		}
		cmd := "mdbook-" + name
		if tbl, ok := raw.(map[string]any); ok {
			if v, ok := tbl["command"].(string); ok && v != "" {
				cmd = v
			}
		}
		out[name] = NewCmdRenderer(name, cmd)
	}
	return out, nil
}

// ShouldRunPreprocessor mirrors
// crates/mdbook-driver/src/mdbook.rs::preprocessor_should_run.
func ShouldRunPreprocessor(p plugin.Preprocessor, renderer plugin.Renderer, cfg *model.Config) (bool, error) {
	name := p.Name()
	// Built-in preprocessors default to always running.
	if contains(defaultPreprocessors, name) && cfg.Build.UseDefaultPreprocessors {
		return p.SupportsRenderer(renderer.Name())
	}
	// Otherwise consult the user-configured renderers whitelist.
	if entry, ok := parsePreprocessorConfig(cfg)[name]; ok && len(entry.Renderers) > 0 {
		for _, r := range entry.Renderers {
			if r == renderer.Name() {
				return true, nil
			}
		}
		return false, nil
	}
	return p.SupportsRenderer(renderer.Name())
}

func contains(haystack []string, needle string) bool {
	for _, x := range haystack {
		if x == needle {
			return true
		}
	}
	return false
}

// RunPreprocessors is a convenience that runs the preprocessor chain against
// a book, threading the PreprocessorContext through so chapter_titles
// accumulate correctly.
func RunPreprocessors(pre []plugin.Preprocessor, cfg *model.Config, root string, renderer string, b *model.Book) (*model.Book, *plugin.PreprocessorContext, error) {
	wc := plugin.ToWireConfig(cfg)
	ctx := &plugin.PreprocessorContext{
		Root:          root,
		Config:        plugin.NewPreprocessorConfig(wc),
		Renderer:      renderer,
		MdbookVersion: plugin.MdbookVersion,
		ChapterTitles: map[string]string{},
	}
	current := b
	for _, p := range pre {
		next, err := p.Run(ctx, current)
		if err != nil {
			return nil, nil, fmt.Errorf("preprocessor %q: %w", p.Name(), err)
		}
		current = next
	}
	return current, ctx, nil
}
