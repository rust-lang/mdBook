package plugin

import (
	"testing"

	"mdbook-go/internal/config"
)

func preprocessorNames(pre []Preprocessor) []string {
	names := make([]string, 0, len(pre))
	for _, p := range pre {
		names = append(names, p.Name())
	}
	return names
}

func requireNames(t *testing.T, got, want []string) {
	t.Helper()
	if len(got) != len(want) {
		t.Fatalf("got names %v, want %v", got, want)
	}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("got names %v, want %v", got, want)
		}
	}
}

func TestBuildPreprocessorsDefaultTerminates(t *testing.T) {
	pre, err := BuildPreprocessors(config.New(), ".")
	if err != nil {
		t.Fatal(err)
	}
	requireNames(t, preprocessorNames(pre), []string{"index", "links"})
}

func TestBuildPreprocessorsBeforeAfterOrdering(t *testing.T) {
	cfg := config.New()
	cfg.Build.UseDefaultPreprocessors = false
	cfg.Preprocessor = map[string]any{
		"first": map[string]any{
			"before": "second",
		},
		"second": map[string]any{},
		"third": map[string]any{
			"after": "second",
		},
	}

	pre, err := BuildPreprocessors(cfg, ".")
	if err != nil {
		t.Fatal(err)
	}
	requireNames(t, preprocessorNames(pre), []string{"first", "second", "third"})
}
