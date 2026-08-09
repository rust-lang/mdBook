package model

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"gopkg.in/yaml.v3"
)

// ConfigFileName is the book configuration file read by the loader and
// written by create. The Rust baseline still reads book.toml (TOML); this
// YAML file is the Go-side equivalent, with the same keys.
const ConfigFileName = "doclens.yaml"

// Load reads a doclens.yaml file from disk and returns a populated Config.
// Dynamic sections (output.* and preprocessor.*) are retained as raw yaml
// values so plugins can decode them with their own schema.
func Load(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read %s: %w", path, err)
	}
	cfg := New()
	if err := yaml.Unmarshal(data, cfg); err != nil {
		return nil, fmt.Errorf("parse %s: %w", path, err)
	}
	return cfg, nil
}

// LoadBook is a convenience that resolves doclens.yaml inside root.
func LoadBook(root string) (*Config, error) {
	return Load(filepath.Join(root, ConfigFileName))
}

// PackageConfig is the [package] section.
type PackageConfig struct {
	Title         string `yaml:"title"`
	Description   string `yaml:"description"`
	Language      string `yaml:"language"`
	TextDirection string `yaml:"text-direction"`
	Root          string `yaml:"root"`
}

// BuildConfig is the [build] section.
type BuildConfig struct {
	BuildDir                string   `yaml:"build-dir"`
	ExtraWatchDirs          []string `yaml:"extra-watch-dirs"`
	CreateMissing           bool     `yaml:"create-missing"`
	PreRender               []string `yaml:"pre-render"`
	UseDefaultPreprocessors bool     `yaml:"use-default-preprocessors"`
}

// HtmlConfig is the [output.html] section. Field defaults are supplied by
// DefaultHTML rather than the Go zero value, because most of the Rust booleans
// default to true.
type HtmlConfig struct {
	Theme              string `yaml:"theme"`
	DefaultTheme       string `yaml:"default-theme"`
	PreferredDarkTheme string `yaml:"preferred-dark-theme"`
	// Mode selects the keyboard navigation script emitted into every page:
	// "vim" (h/l navigate chapters, default) or "normal" (arrows only).
	Mode             string   `yaml:"mode"`
	SmartPunctuation bool     `yaml:"smart-punctuation"`
	DefinitionLists  bool     `yaml:"definition-lists"`
	Admonitions      bool     `yaml:"admonitions"`
	MathJaxSupport   bool     `yaml:"mathjax-support"`
	AdditionalCSS    []string `yaml:"additional-css"`
	AdditionalJS     []string `yaml:"additional-js"`

	Fold Fold `yaml:"fold"`
	Code Code `yaml:"code"`

	NoSectionLabel bool    `yaml:"no-section-label"`
	Search         *Search `yaml:"search"`

	GitRepositoryURL  string `yaml:"git-repository-url"`
	GitRepositoryIcon string `yaml:"git-repository-icon"`
	EditURLTemplate   string `yaml:"edit-url-template"`

	// Input404 is a pointer so an explicit empty string, which disables the
	// page, can be told apart from the key being absent.
	Input404 *string `yaml:"input-404"`
	SiteURL  string  `yaml:"site-url"`
	CName    string  `yaml:"cname"`

	// LiveReloadEndpoint is set by `serve`, never read from doclens.yaml.
	LiveReloadEndpoint string `yaml:"-"`

	Redirect         map[string]string `yaml:"redirect"`
	HashFiles        bool              `yaml:"hash-files"`
	SidebarHeaderNav bool              `yaml:"sidebar-header-nav"`
}

// SearchChapter is a per-chapter search override in [output.html.search.chapter].
type SearchChapter struct {
	Enable bool `yaml:"enable"`
}

// Search is the [output.html.search] section.
type Search struct {
	Enable            bool                     `yaml:"enable"`
	LimitResults      int                      `yaml:"limit-results"`
	TeaserWordCount   int                      `yaml:"teaser-word-count"`
	UseBooleanAnd     bool                     `yaml:"use-boolean-and"`
	BoostTitle        int                      `yaml:"boost-title"`
	BoostHierarchy    int                      `yaml:"boost-hierarchy"`
	BoostParagraph    int                      `yaml:"boost-paragraph"`
	Expand            bool                     `yaml:"expand"`
	HeadingSplitLevel int                      `yaml:"heading-split-level"`
	CopyJS            bool                     `yaml:"copy-js"`
	Chapter           map[string]SearchChapter `yaml:"chapter"`
}

// ChaptersConfig is the [chapters] section: the book's table of contents.
// It replaces SUMMARY.md — the three lists mirror the mdBook summary grammar:
// prefix chapters precede the numbered list, the numbered list carries the
// section numbering, and suffix chapters follow it.
type ChaptersConfig struct {
	Prefix   []ChapterItem `yaml:"prefix"`
	Numbered []ChapterItem `yaml:"numbered"`
	Suffix   []ChapterItem `yaml:"suffix"`
}

// ChapterItem is one entry in a chapters list. Exactly one form applies: a
// chapter has Name/Path (Path "" marks a draft chapter that still consumes a
// number), a part title has Part set, and a separator has Separator set.
type ChapterItem struct {
	Name      string        `yaml:"name"`
	Path      string        `yaml:"path"`
	Children  []ChapterItem `yaml:"children"`
	Part      string        `yaml:"part"`
	Separator bool          `yaml:"separator"`
}

// Config is the full effective configuration loaded from doclens.yaml.
// Output and Preprocessor are kept as generic maps so plugin configurations
// stay dynamic.
type Config struct {
	Package      PackageConfig  `yaml:"package"`
	Build        BuildConfig    `yaml:"build"`
	Chapters     ChaptersConfig `yaml:"chapters"`
	Output       map[string]any `yaml:"output"`
	Preprocessor map[string]any `yaml:"preprocessor"`
}

// New returns a Config populated with the doclens defaults. These deviate
// from mdbook-core: doclens uses docs/ for sources and .doclens/ for the
// build output (Rust mdbook defaults to src/ and book/).
func New() *Config {
	return &Config{
		Package: PackageConfig{
			Root: "docs",
		},
		Build: BuildConfig{
			BuildDir:                ".doclens",
			CreateMissing:           true,
			UseDefaultPreprocessors: true,
		},
		Output:       map[string]any{},
		Preprocessor: map[string]any{},
	}
}

// SetRoot resolves the source root directory relative to the book root.
func (c *Config) SetRoot(root string) {
	if c.Package.Root == "" {
		c.Package.Root = "docs"
	}
	if !filepathIsAbs(c.Package.Root) {
		c.Package.Root = joinPath(root, c.Package.Root)
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

// ToJSON serialises the config to JSON for plugin hand-off.
func (c *Config) ToJSON() ([]byte, error) {
	type alias Config
	return json.Marshal((*alias)(c))
}
