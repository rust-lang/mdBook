package model

import (
	"path/filepath"
	"strings"

	"gopkg.in/yaml.v3"
)

// Fold is [output.html.fold].
type Fold struct {
	Enable bool  `yaml:"enable"`
	Level  uint8 `yaml:"level"`
}

// Code is [output.html.code].
type Code struct {
	// HideLines maps a language to the prefix marking a hidden line.
	HideLines map[string]string `yaml:"hidelines"`
}

// DefaultSearch returns the Rust defaults for [output.html.search].
func DefaultSearch() Search {
	return Search{
		Enable:            true,
		LimitResults:      30,
		TeaserWordCount:   30,
		UseBooleanAnd:     false,
		BoostTitle:        2,
		BoostHierarchy:    1,
		BoostParagraph:    1,
		Expand:            true,
		HeadingSplitLevel: 3,
		CopyJS:            true,
		Chapter:           map[string]SearchChapter{},
	}
}

// DefaultHTML returns [output.html] with the same defaults as
// `impl Default for HtmlConfig` in crates/mdbook-core/src/config.rs.
func DefaultHTML() *HtmlConfig {
	return &HtmlConfig{
		Mode:             "vim",
		SmartPunctuation: true,
		DefinitionLists:  true,
		Admonitions:      true,
		MathJaxSupport:   false,
		Fold:             Fold{},
		Code:             Code{HideLines: map[string]string{}},
		NoSectionLabel:   false,
		Redirect:         map[string]string{},
		HashFiles:        true,
		SidebarHeaderNav: true,
	}
}

// HTML decodes [output.html] on top of the Rust defaults. An absent table
// yields the defaults unchanged.
func (c *Config) HTML() (*HtmlConfig, error) {
	cfg := DefaultHTML()
	raw, ok := c.Output["html"]
	if !ok {
		return cfg, nil
	}
	// Round-trip through YAML so absent keys keep their default rather than
	// being zeroed, which is what serde's #[serde(default)] does.
	encoded, err := yaml.Marshal(map[string]any{"html": raw})
	if err != nil {
		return nil, err
	}
	var wrapper struct {
		HTML *HtmlConfig `yaml:"html"`
	}
	wrapper.HTML = cfg
	if err := yaml.Unmarshal(encoded, &wrapper); err != nil {
		return nil, err
	}
	// `search` is absent unless the user wrote the table, but once written the
	// unspecified keys still take the Rust defaults.
	if cfg.Search != nil {
		merged := DefaultSearch()
		if sub, ok := subTable(raw, "search"); ok {
			encoded, err := yaml.Marshal(map[string]any{"search": sub})
			if err != nil {
				return nil, err
			}
			var sw struct {
				Search *Search `yaml:"search"`
			}
			sw.Search = &merged
			if err := yaml.Unmarshal(encoded, &sw); err != nil {
				return nil, err
			}
		}
		cfg.Search = &merged
	}
	return cfg, nil
}

// subTable pulls a nested table out of a dynamic [output.html] value.
func subTable(raw any, key string) (any, bool) {
	table, ok := raw.(map[string]any)
	if !ok {
		return nil, false
	}
	value, ok := table[key]
	return value, ok
}

// EffectiveSearch returns the search settings actually in force, which are the
// defaults when the user did not write an [output.html.search] table.
func (h *HtmlConfig) EffectiveSearch() Search {
	if h.Search == nil {
		return DefaultSearch()
	}
	return *h.Search
}

// Get404OutputFile returns the file name of the 404 page: `input-404` with its
// `.md` suffix replaced, defaulting to "404.html". An empty `input-404`
// disables the page entirely, which callers check separately.
func (h *HtmlConfig) Get404OutputFile() string {
	input := "404.md"
	if h.Input404 != nil {
		input = *h.Input404
	}
	return strings.ReplaceAll(input, ".md", ".html")
}

// Render404 reports whether the 404 page should be written at all.
func (h *HtmlConfig) Render404() bool {
	return h.Input404 == nil || *h.Input404 != ""
}

// ThemeDir resolves the theme directory against the book root.
func (h *HtmlConfig) ThemeDir(root string) string {
	if h.Theme == "" {
		return filepath.Join(root, "theme")
	}
	if filepath.IsAbs(h.Theme) {
		return h.Theme
	}
	return filepath.Join(root, h.Theme)
}

// DefaultThemeName returns the light theme name used by the templates.
func (h *HtmlConfig) DefaultThemeName() string {
	if h.DefaultTheme == "" {
		return "light"
	}
	return strings.ToLower(h.DefaultTheme)
}

// PreferredDarkThemeName returns the dark theme name used by the templates.
func (h *HtmlConfig) PreferredDarkThemeName() string {
	if h.PreferredDarkTheme == "" {
		return "navy"
	}
	return strings.ToLower(h.PreferredDarkTheme)
}

// GitRepositoryIconName returns the FontAwesome icon for the repository link.
func (h *HtmlConfig) GitRepositoryIconName() string {
	if h.GitRepositoryIcon == "" {
		return "fab-github"
	}
	return h.GitRepositoryIcon
}

// GitRepositoryIconClass maps the icon's style prefix to a FontAwesome style.
// The prefix is the segment before the first `-`, matched exactly, as
// make_data does in crates/mdbook-html/src/html_handlebars/hbs_renderer.rs.
func (h *HtmlConfig) GitRepositoryIconClass() string {
	prefix, _, _ := strings.Cut(h.GitRepositoryIconName(), "-")
	switch prefix {
	case "fas":
		return "solid"
	case "fab":
		return "brands"
	default:
		return "regular"
	}
}

// RealizedTextDirection returns "ltr" or "rtl", inferring the direction from
// the book language when it was not set explicitly. The language list is copied
// from TextDirection::from_lang_code in crates/mdbook-core/src/config.rs.
func (b *PackageConfig) RealizedTextDirection() string {
	if b.TextDirection != "" {
		return b.TextDirection
	}
	switch b.Language {
	case "ar", "ara", "arc", "ae", "ave", "egy", "he", "heb", "nqo", "pal",
		"phn", "sam", "syc", "syr", "fa", "per", "fas", "ku", "kur", "ur",
		"urd", "pus", "ps", "yi", "yid":
		return "rtl"
	default:
		return "ltr"
	}
}
