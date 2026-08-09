package runner

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"mdbook-go/internal/model"
	"mdbook-go/internal/plugin"
)

// IndexPreprocessor rewrites chapters whose source file is named `README.md`
// (case-insensitive) so they render as `index.html`. It mirrors
// crates/mdbook-driver/src/builtin_preprocessors/index.rs.
type IndexPreprocessor struct{}

// Name matches Rust's IndexPreprocessor::NAME.
func (IndexPreprocessor) Name() string { return "index" }

// Run rewrites every chapter with a README-style path to `index.md`.
func (p IndexPreprocessor) Run(ctx *plugin.PreprocessorContext, b *model.Book) (*model.Book, error) {
	srcDir := ctx.Config.Package.Root
	b.Iter(func(ch *model.Chapter) bool {
		if ch.IsDraft() {
			return true
		}
		stem := strings.TrimSuffix(filepath.Base(ch.Path), filepath.Ext(ch.Path))
		if !readmeRegexp.MatchString(stem) {
			return true
		}
		indexPath := filepath.Join(filepath.Dir(ch.Path), "index.md")
		abs := filepath.Join(srcDir, indexPath)
		if _, err := os.Stat(abs); err == nil {
			fmt.Fprintf(os.Stderr,
				"warning: It seems that there are both %s and index.md under %q.\n"+
					"warning: mdbook converts %s into index.html by default. It may cause\n"+
					"warning: unexpected behavior if putting both files under the same directory.\n"+
					"warning: To solve the warning, try to rearrange the book structure or disable\n"+
					"warning: \"index\" preprocessor to stop the conversion.\n",
				filepath.Base(ch.Path), filepath.Dir(ch.Path), filepath.Base(ch.Path))
		}
		ch.Path = indexPath
		// SourcePath intentionally preserved — the edit URL and `git_repository_edit_url`
		// template need the original README.md filename, not the rewritten index.md.
		return true
	})
	return b, nil
}

// SupportsRenderer is true for all renderers.
func (IndexPreprocessor) SupportsRenderer(string) (bool, error) { return true, nil }

var readmeRegexp = regexp.MustCompile(`(?i)^readme$`)
