package driver

import (
	"fmt"
	"os"
	"path/filepath"

	"mdbook-go/internal/theme"
)

// InitOptions configures a single Init call. It is the Go analogue of the
// clap arguments on src/cmd/init.rs::make_subcommand.
//
// The zero value is meaningful: an empty Title produces the Rust-equivalent
// default "My Book", an empty Ignore produces a .gitignore, and Theme=false
// skips theme copy. Callers that want strict Rust parity should set Title
// explicitly when Force is true (Rust emits a `title = ""` line in that case;
// we instead keep "My Book" so the resulting book.toml stays valid).
type InitOptions struct {
	// Title is the book title written to book.toml. "" defaults to
	// "My Book" so the generated config is usable without further edits.
	Title string
	// Theme copies the embedded default theme into <root>/theme/.
	Theme bool
	// Force skips interactive confirmation prompts. The M4.1 Go port
	// does not prompt for anything yet, so Force is currently unused
	// beyond plumbing; it is accepted so the CLI surface matches Rust.
	Force bool
	// Ignore controls whether a .gitignore file is created. Accepted
	// values: "git" (write a gitignore, the default) or "none" (skip).
	// Empty string is treated as "git" for compatibility with the
	// M1-era behaviour.
	Ignore string
}

// Init creates a fresh book skeleton at root. It mirrors
// `crates/mdbook/src/cmd/init.rs::execute` end-to-end:
//
//   - <root>/src/                directory
//   - <root>/book.toml           minimal config (with the chosen title)
//   - <root>/src/SUMMARY.md      two-chapter skeleton
//   - <root>/src/intro.md        intro chapter
//   - <root>/src/chapter_1.md    first numbered chapter with a Rust code block
//   - <root>/.gitignore          "book/\n"  (skipped when Ignore == "none")
//   - <root>/theme/              embedded default theme (only when opts.Theme)
func Init(root string, opts InitOptions) error {
	if err := os.MkdirAll(filepath.Join(root, "src"), 0o755); err != nil {
		return err
	}
	title := opts.Title
	if title == "" {
		title = "My Book"
	}
	bookToml := fmt.Sprintf(
		"[book]\ntitle = %q\nauthors = []\nlanguage = \"en\"\nsrc = \"src\"\n\n[build]\nbuild-dir = \"book\"\ncreate-missing = true\n",
		title,
	)
	if err := os.WriteFile(filepath.Join(root, "book.toml"), []byte(bookToml), 0o644); err != nil {
		return err
	}
	summary := "# Summary\n\n[Introduction](intro.md)\n\n# Chapter 1\n\n- [Chapter 1](chapter_1.md)\n"
	if err := os.WriteFile(filepath.Join(root, "src", "SUMMARY.md"), []byte(summary), 0o644); err != nil {
		return err
	}
	intro := "# Introduction\n\nWelcome to **mdbook-go**.\n"
	if err := os.WriteFile(filepath.Join(root, "src", "intro.md"), []byte(intro), 0o644); err != nil {
		return err
	}
	c1 := "# Chapter 1\n\nFirst chapter content.\n\n```rust\nfn main() { println!(\"hi\"); }\n```\n"
	if err := os.WriteFile(filepath.Join(root, "src", "chapter_1.md"), []byte(c1), 0o644); err != nil {
		return err
	}
	if opts.Ignore != "none" {
		gitignore := "book/\n"
		if err := os.WriteFile(filepath.Join(root, ".gitignore"), []byte(gitignore), 0o644); err != nil {
			return err
		}
	}
	if opts.Theme {
		themeDir := filepath.Join(root, "theme")
		if err := os.MkdirAll(themeDir, 0o755); err != nil {
			return err
		}
		// printEnable=true matches Rust's `MDBook::init(...).copy_theme(true)`,
		// which is the default in src/cmd/init.rs when --theme is given. It
		// controls whether css/print.css is written into the theme dir.
		if err := theme.Copy(themeDir, true); err != nil {
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