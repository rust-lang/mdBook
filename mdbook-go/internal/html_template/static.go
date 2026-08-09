// This file assembles, fingerprints and writes the theme's static assets.
// It is a port of crates/mdbook-html/src/html_handlebars/static_files.rs.
package html_template

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"mdbook-go/internal/model"
	"mdbook-go/pkg/fs"
)

// resourcePattern matches the `{{ resource "name" }}` placeholders that appear
// inside CSS and JS assets. They are substituted at write time rather than by
// the template engine.
var resourcePattern = regexp.MustCompile(`\{\{ resource "([^"]+)" \}\}`)

// file is one asset to emit. Exactly one of data and sourcePath is set.
type file struct {
	// name is the destination path relative to the output root, before hashing.
	name string
	// data holds the contents of a built-in asset.
	data []byte
	// sourcePath is set for user-supplied ("additional") assets, which are
	// streamed from disk.
	sourcePath string
	// hashedName is filled in by Hash.
	hashedName string
}

func (f *file) isBuiltin() bool { return f.sourcePath == "" }

func (f *file) destination() string {
	if f.hashedName != "" {
		return f.hashedName
	}
	return f.name
}

// Files is the ordered collection of assets for one build.
type Files struct {
	files     []*file
	hashFiles bool
	// resources maps a logical asset name to its emitted (possibly hashed)
	// name. It is populated by Write.
	resources map[string]string
}

// New builds the asset set for the given theme and configuration, mirroring
// StaticFiles::new. root is the book root, used to resolve additional-css and
// additional-js paths.
func NewStaticFiles(t *Theme, cfg *model.HtmlConfig, root string) (*Files, error) {
	f := &Files{hashFiles: cfg.HashFiles, resources: map[string]string{}}

	f.AddBuiltin("book.js", t.JS)
	f.AddBuiltin("css/general.css", t.GeneralCSS)
	f.AddBuiltin("css/chrome.css", t.ChromeCSS)
	f.AddBuiltin("css/variables.css", t.VariablesCSS)
	f.AddBuiltin("highlight.css", t.HighlightCSS)
	f.AddBuiltin("tomorrow-night.css", t.TomorrowNightCSS)
	f.AddBuiltin("ayu-highlight.css", t.AyuHighlightCSS)
	f.AddBuiltin("highlight.min.js", t.HighlightJS)
	f.AddBuiltin("clipboard.min.js", t.ClipboardJS)
	// Keyboard navigation variant selected by [output.html.mode]. Both variants
	// are emitted under distinct names; the template references only the one
	// matching the configured mode.
	f.AddBuiltin("nav-vim.js", t.NavVimJS)
	f.AddBuiltin("nav-normal.js", t.NavNormalJS)
	f.AddBuiltin("css/github-markdown-light.css", t.GitHubMarkdownLightCSS)
	f.AddBuiltin("css/github-markdown-dark.css", t.GitHubMarkdownDarkCSS)

	for _, custom := range append(append([]string{}, cfg.AdditionalCSS...), cfg.AdditionalJS...) {
		f.addAdditional(custom, filepath.Join(root, filepath.FromSlash(custom)))
	}
	return f, nil
}

// AddBuiltin appends an in-memory asset. The renderer uses it for the generated
// toc.js and search assets.
func (f *Files) AddBuiltin(name string, data []byte) {
	f.files = append(f.files, &file{name: name, data: data})
}

func (f *Files) addAdditional(name, sourcePath string) {
	f.files = append(f.files, &file{name: name, sourcePath: sourcePath})
}

// Hash fingerprints every eligible asset, renaming `name.ext` to
// `name-<8 hex>.ext`. Built-in `.txt` assets (the font licences) are never
// renamed. Does nothing when hash-files is off.
func (f *Files) Hash() error {
	if !f.hashFiles {
		return nil
	}
	for _, file := range f.files {
		name, suffix, found := strings.Cut(file.name, ".")
		if !found || name == "" || suffix == "" {
			continue
		}
		if file.isBuiltin() && suffix == "txt" {
			continue
		}
		var sum []byte
		if file.isBuiltin() {
			digest := sha256.Sum256(file.data)
			sum = digest[:]
		} else {
			digest, err := hashFileOnDisk(file.sourcePath)
			if err != nil {
				return err
			}
			sum = digest
		}
		file.hashedName = fmt.Sprintf("%s-%s.%s", name, hex.EncodeToString(sum[:4]), suffix)
	}
	return nil
}

func hashFileOnDisk(path string) ([]byte, error) {
	in, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer in.Close()
	digest := sha256.New()
	if _, err := io.Copy(digest, in); err != nil {
		return nil, err
	}
	return digest.Sum(nil), nil
}

// Resolve returns the emitted name of a logical asset, or the name itself when
// it was not hashed or not present.
func (f *Files) Resolve(name string) string {
	if resolved, ok := f.resources[name]; ok {
		return resolved
	}
	for _, file := range f.files {
		if file.name == name {
			return file.destination()
		}
	}
	return name
}

// Write emits every asset into destination and returns the logical-name to
// emitted-name map used by the `resource` template helper. CSS and JS files get
// their `{{ resource "..." }}` placeholders substituted first.
func (f *Files) Write(destination string) (map[string]string, error) {
	for _, file := range f.files {
		f.resources[file.name] = file.destination()
	}

	for _, file := range f.files {
		out := filepath.Join(destination, filepath.FromSlash(file.destination()))
		if err := os.MkdirAll(filepath.Dir(out), 0o755); err != nil {
			return nil, err
		}
		rewritable := strings.HasSuffix(file.destination(), ".css") ||
			strings.HasSuffix(file.destination(), ".js")

		if !rewritable {
			if file.isBuiltin() {
				if err := os.WriteFile(out, file.data, 0o644); err != nil {
					return nil, err
				}
			} else if err := fs.CopyFile(file.sourcePath, out); err != nil {
				return nil, err
			}
			continue
		}

		data := file.data
		if !file.isBuiltin() {
			read, err := os.ReadFile(file.sourcePath)
			if err != nil {
				return nil, err
			}
			data = read
		}
		if err := os.WriteFile(out, f.replaceAll(data, file.destination()), 0o644); err != nil {
			return nil, err
		}
	}
	return f.resources, nil
}

// replaceAll substitutes the `{{ resource "name" }}` placeholders in an asset,
// prefixing each with enough `../` to get back to the output root.
func (f *Files) replaceAll(data []byte, destination string) []byte {
	pathToRoot := fs.PathToRoot(destination)
	return resourcePattern.ReplaceAllFunc(data, func(match []byte) []byte {
		name := string(resourcePattern.FindSubmatch(match)[1])
		return []byte(pathToRoot + f.Resolve(name))
	})
}
