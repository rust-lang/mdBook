// Package config holds the typed and dynamic configuration objects that drive a
// book build. The shapes are deliberately compatible with the structures
// produced by mdbook-core so plugin JSON remains stable.
package config

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
)

// BookConfig is the [book] section.
type BookConfig struct {
	Title         string   `toml:"title"`
	Authors       []string `toml:"authors"`
	Description   string   `toml:"description"`
	Language      string   `toml:"language"`
	TextDirection string   `toml:"text-direction"`
	Multilingual  bool     `toml:"multilingual"`
	SourceDir     string   `toml:"src"`
}

// BuildConfig is the [build] section.
type BuildConfig struct {
	BuildDir                string   `toml:"build-dir"`
	ExtraWatchDirs          []string `toml:"extra-watch-dirs"`
	CreateMissing           bool     `toml:"create-missing"`
	PreRender               []string `toml:"pre-render"`
	UseDefaultPreprocessors bool     `toml:"use-default-preprocessors"`
}

// RustConfig is the [rust] section.
type RustConfig struct {
	Edition     string `toml:"edition"`
	Description string `toml:"description"`
}

// HtmlConfig is the [output.html] section. Field defaults are supplied by
// DefaultHTML rather than the Go zero value, because most of the Rust booleans
// default to true.
type HtmlConfig struct {
	Theme              string   `toml:"theme"`
	DefaultTheme       string   `toml:"default-theme"`
	PreferredDarkTheme string   `toml:"preferred-dark-theme"`
	SmartPunctuation   bool     `toml:"smart-punctuation"`
	DefinitionLists    bool     `toml:"definition-lists"`
	Admonitions        bool     `toml:"admonitions"`
	MathJaxSupport     bool     `toml:"mathjax-support"`
	AdditionalCSS      []string `toml:"additional-css"`
	AdditionalJS       []string `toml:"additional-js"`

	Fold       Fold       `toml:"fold"`
	Code       Code       `toml:"code"`

	NoSectionLabel bool    `toml:"no-section-label"`
	Search         *Search `toml:"search"`

	GitRepositoryURL  string `toml:"git-repository-url"`
	GitRepositoryIcon string `toml:"git-repository-icon"`
	EditURLTemplate   string `toml:"edit-url-template"`

	// Input404 is a pointer so an explicit empty string, which disables the
	// page, can be told apart from the key being absent.
	Input404 *string `toml:"input-404"`
	SiteURL  string  `toml:"site-url"`
	CName    string  `toml:"cname"`

	// LiveReloadEndpoint is set by `serve`, never read from book.toml.
	LiveReloadEndpoint string `toml:"-"`

	Redirect         map[string]string `toml:"redirect"`
	HashFiles        bool              `toml:"hash-files"`
	SidebarHeaderNav bool              `toml:"sidebar-header-nav"`
}

// SearchChapter is a per-chapter search override in [output.html.search.chapter].
type SearchChapter struct {
	Enable bool `toml:"enable"`
}

// Search is the [output.html.search] section.
type Search struct {
	Enable            bool                     `toml:"enable"`
	LimitResults      int                      `toml:"limit-results"`
	TeaserWordCount   int                      `toml:"teaser-word-count"`
	UseBooleanAnd     bool                     `toml:"use-boolean-and"`
	BoostTitle        int                      `toml:"boost-title"`
	BoostHierarchy    int                      `toml:"boost-hierarchy"`
	BoostParagraph    int                      `toml:"boost-paragraph"`
	Expand            bool                     `toml:"expand"`
	HeadingSplitLevel int                      `toml:"heading-split-level"`
	CopyJS            bool                     `toml:"copy-js"`
	Chapter           map[string]SearchChapter `toml:"chapter"`
}

// Config is the full effective configuration loaded from book.toml and env.
// Output and Preprocessor are kept as generic maps so plugin configurations
// stay dynamic.
type Config struct {
	Book         BookConfig     `toml:"book"`
	Build        BuildConfig    `toml:"build"`
	Rust         RustConfig     `toml:"rust"`
	Output       map[string]any `toml:"output"`
	Preprocessor map[string]any `toml:"preprocessor"`
}

// New returns a Config populated with the same defaults as mdbook-core.
func New() *Config {
	return &Config{
		Book: BookConfig{
			SourceDir: "src",
		},
		Build: BuildConfig{
			BuildDir:                "book",
			CreateMissing:           true,
			UseDefaultPreprocessors: true,
		},
		Output:       map[string]any{},
		Preprocessor: map[string]any{},
	}
}

// SetSourceDir resolves the book source directory relative to the book root.
func (c *Config) SetSourceDir(root string) {
	if c.Book.SourceDir == "" {
		c.Book.SourceDir = "src"
	}
	if !filepathIsAbs(c.Book.SourceDir) {
		c.Book.SourceDir = joinPath(root, c.Book.SourceDir)
	}
}

func filepathIsAbs(p string) bool {
	return len(p) > 0 && p[0] == '/'
}

func joinPath(a, b string) string {
	if a == "" {
		return b
	}
	if strings.HasSuffix(a, "/") {
		return a + b
	}
	return a + "/" + b
}

// FromEnv applies MDBOOK_* overrides. The variable name after MDBOOK_ is a dot
// path (e.g. MDBOOK_BOOK_TITLE -> book.title). Numbers and booleans are parsed
// when possible; otherwise the value is stored as a string.
func (c *Config) FromEnv() error {
	for _, kv := range os.Environ() {
		eq := strings.IndexByte(kv, '=')
		if eq < 0 {
			continue
		}
		name, value := kv[:eq], kv[eq+1:]
		if !strings.HasPrefix(name, "MDBOOK_") {
			continue
		}
		path := strings.ToLower(strings.TrimPrefix(name, "MDBOOK_"))
		path = strings.ReplaceAll(path, "__", ".")
		if err := c.setPath(path, value); err != nil {
			return fmt.Errorf("env %s: %w", name, err)
		}
	}
	return nil
}

// setPath writes a value at the given dotted path into the strongly typed
// fields, falling back to dynamic output/preprocessor tables.
func (c *Config) setPath(path, value string) error {
	switch path {
	case "book.title":
		c.Book.Title = value
	case "book.description":
		c.Book.Description = value
	case "book.language":
		c.Book.Language = value
	case "build.build-dir":
		c.Build.BuildDir = value
	case "build.create-missing":
		c.Build.CreateMissing = parseBool(value)
	case "rust.edition":
		c.Rust.Edition = value
	default:
		// Unknown paths are left untouched; plugin configs are dynamic.
	}
	return nil
}

func parseBool(s string) bool {
	switch strings.ToLower(s) {
	case "1", "true", "yes", "on":
		return true
	}
	return false
}

// ToJSON serialises the config to JSON for plugin hand-off.
func (c *Config) ToJSON() ([]byte, error) {
	type alias Config
	return json.Marshal((*alias)(c))
}
