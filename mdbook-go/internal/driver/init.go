package driver

import (
	"fmt"
	"os"
	"path/filepath"
)

// Init creates a fresh book skeleton at root. The optional copyTheme flag
// controls whether the default theme is copied under theme/.
func Init(root string, copyTheme bool) error {
	if err := os.MkdirAll(filepath.Join(root, "src"), 0o755); err != nil {
		return err
	}
	bookToml := "[book]\ntitle = \"My Book\"\nauthors = []\nlanguage = \"en\"\nsrc = \"src\"\n\n[build]\nbuild-dir = \"book\"\ncreate-missing = true\n"
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
	gitignore := "book/\n"
	if err := os.WriteFile(filepath.Join(root, ".gitignore"), []byte(gitignore), 0o644); err != nil {
		return err
	}
	if copyTheme {
		// Theme is a no-op for M1; placeholder directory.
		if err := os.MkdirAll(filepath.Join(root, "theme"), 0o755); err != nil {
			return err
		}
		if err := os.WriteFile(filepath.Join(root, "theme", "README.md"), []byte("# Theme\n\nCustom theme files go here.\n"), 0o644); err != nil {
			return err
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
