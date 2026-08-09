package runner

import (
	"fmt"
	"os"
	"path/filepath"

	htmltpl "mdbook-go/internal/html_template"
	"mdbook-go/internal/model"
)

// InitOptions configures a single Init call. It is the Go analogue of the
// clap arguments on src/cmd/init.rs::make_subcommand, minus the interactive
// prompts (Go never prompts) and the .gitignore generation (doclens does
// not create one; the watch gitignore parser still honours a user-provided
// .gitignore, see pkg/cmd/watch/gitignore.go).
//
// The zero value is meaningful: an empty Title produces the Rust-equivalent
// default "My Book" and Theme=false skips theme copy.
type InitOptions struct {
	// Title is the book title written to doclens.yaml. "" defaults to
	// "My Book" so the generated config is usable without further edits.
	Title string
	// Theme copies the embedded default theme into <root>/theme/.
	Theme bool
	// Force skips interactive confirmation prompts. The Go port does not
	// prompt for anything, so Force is currently unused beyond plumbing;
	// it is accepted so the CLI surface matches Rust.
	Force bool
}

// Init creates a fresh book skeleton at root. It mirrors
// `crates/mdbook/src/cmd/init.rs::execute` end-to-end (with doclens
// deviations: sources go to docs/ instead of src/, no .gitignore, and
// the build directory default is .doclens/):
//
//   - <root>/docs/                directory
//   - <root>/doclens.yaml         minimal config (with the chosen title and a
//                                 [chapters] skeleton in place of SUMMARY.md)
//   - <root>/docs/intro.md        intro chapter
//   - <root>/docs/chapter_1.md    first numbered chapter with a sample code block
//   - <root>/theme/               embedded default theme (only when opts.Theme)
func Init(root string, opts InitOptions) error {
	if err := os.MkdirAll(filepath.Join(root, "docs"), 0o755); err != nil {
		return err
	}
	title := opts.Title
	if title == "" {
		title = "My Book"
	}
	doclensYAML := fmt.Sprintf(
		"package:\n  title: %q\n  language: en\n  root: docs\n\nbuild:\n  build-dir: .doclens\n  create-missing: true\n\nchapters:\n  prefix:\n    - name: Introduction\n      path: intro.md\n  numbered:\n    - name: Chapter 1\n      path: chapter_1.md\n",
		title,
	)
	if err := os.WriteFile(filepath.Join(root, model.ConfigFileName), []byte(doclensYAML), 0o644); err != nil {
		return err
	}
	intro := "# Introduction\n\nWelcome to **mdbook-go**.\n"
	if err := os.WriteFile(filepath.Join(root, "docs", "intro.md"), []byte(intro), 0o644); err != nil {
		return err
	}
	c1 := "# Chapter 1\n\nFirst chapter content.\n\n```text\nHello, doclens!\n```\n"
	if err := os.WriteFile(filepath.Join(root, "docs", "chapter_1.md"), []byte(c1), 0o644); err != nil {
		return err
	}
	if opts.Theme {
		themeDir := filepath.Join(root, "theme")
		if err := os.MkdirAll(themeDir, 0o755); err != nil {
			return err
		}
		if err := htmltpl.Copy(themeDir); err != nil {
			return fmt.Errorf("copy theme: %w", err)
		}
	}
	return nil
}

// LoadAndBuild is a convenience used by the CLI and the harness.
func LoadAndBuild(root string) error {
	m, err := Load(root)
	if err != nil {
		return fmt.Errorf("load: %w", err)
	}
	return m.Build()
}
