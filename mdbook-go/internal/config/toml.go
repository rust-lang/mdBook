package config

import (
	"fmt"
	"os"
	"path/filepath"

	toml "github.com/pelletier/go-toml/v2"
)

// Load reads a book.toml file from disk and returns a populated Config.
// Dynamic sections ([output.*] and [preprocessor.*]) are retained as raw
// toml values so plugins can decode them with their own schema.
func Load(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read %s: %w", path, err)
	}
	cfg := New()
	if err := toml.Unmarshal(data, cfg); err != nil {
		return nil, fmt.Errorf("parse %s: %w", path, err)
	}
	return cfg, nil
}

// LoadBook is a convenience that resolves book.toml inside root.
func LoadBook(root string) (*Config, error) {
	return Load(filepath.Join(root, "book.toml"))
}
