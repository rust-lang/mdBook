package cmd

import (
	"io"
	"log"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"strings"
	"testing"
)

// TestStaticHandler exercises the file-serving rules implemented in
// staticHandler. It reuses the pre-built fixture under
// mdbook-go/fixtures/serve/book, which contains index.html, intro.html,
// 404.html, and the full hashed asset set.
//
// The cases cover both the original bug (URLs ending in /index.html must
// serve directly, not 301 to ./) and the surrounding behaviours we want
// to preserve (root, regular .html, hashed assets, nested resources,
// 404 fallback).
func TestStaticHandler(t *testing.T) {
	root, err := filepath.Abs("../../tests/serve/book")
	if err != nil {
		t.Fatalf("abs root: %v", err)
	}
	h := staticHandler(root, "404.html", log.New(io.Discard, "", 0))
	srv := httptest.NewServer(h)
	defer srv.Close()

	client := srv.Client()
	// Don't auto-follow redirects; we want to see if any sneak through.
	client.CheckRedirect = func(req *http.Request, via []*http.Request) error {
		return http.ErrUseLastResponse
	}

	cases := []struct {
		name        string
		path        string
		wantStatus  int
		wantBodyHas string
	}{
		{
			name:        "root serves index.html",
			path:        "/",
			wantStatus:  http.StatusOK,
			wantBodyHas: "mdBook",
		},
		{
			name:        "explicit /index.html serves directly (regression)",
			path:        "/index.html",
			wantStatus:  http.StatusOK,
			wantBodyHas: "mdBook",
		},
		{
			name:        "regular chapter",
			path:        "/intro.html",
			wantStatus:  http.StatusOK,
			wantBodyHas: "intro",
		},
		{
			name:        "trailing slash on directory falls back to index.html",
			path:        "/",
			wantStatus:  http.StatusOK,
			wantBodyHas: "mdBook",
		},
		{
			name:        "hashed JS asset",
			path:        "/book-8fa2ce42.js",
			wantStatus:  http.StatusOK,
			wantBodyHas: "mdbook",
		},
		{
			name:        "nested CSS (hashed)",
			path:        "/css/chrome-2b282e04.css",
			wantStatus:  http.StatusOK,
			wantBodyHas: "chrome",
		},
		{
			name:        "404 fallback returns 404 with 404.html body",
			path:        "/nonexistent.html",
			wantStatus:  http.StatusNotFound,
			wantBodyHas: "Document not found",
		},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			resp, err := client.Get(srv.URL + c.path)
			if err != nil {
				t.Fatalf("GET %s: %v", c.path, err)
			}
			defer resp.Body.Close()
			if resp.StatusCode != c.wantStatus {
				t.Errorf("status = %d, want %d", resp.StatusCode, c.wantStatus)
			}
			body, err := io.ReadAll(resp.Body)
			if err != nil {
				t.Fatalf("read body: %v", err)
			}
			if !strings.Contains(strings.ToLower(string(body)), strings.ToLower(c.wantBodyHas)) {
				t.Errorf("body missing %q (got %d bytes)", c.wantBodyHas, len(body))
			}
		})
	}
}

// TestStaticHandler_NoRedirectOnIndex verifies the central regression:
// requesting `/index.html` (or any URL ending in `/index.html`) must
// serve the file directly with 200, not issue a 301. This is the
// Go stdlib bug we work around.
func TestStaticHandler_NoRedirectOnIndex(t *testing.T) {
	root, _ := filepath.Abs("../../tests/serve/book")
	h := staticHandler(root, "404.html", log.New(io.Discard, "", 0))
	srv := httptest.NewServer(h)
	defer srv.Close()

	client := srv.Client()
	client.CheckRedirect = func(req *http.Request, via []*http.Request) error {
		return http.ErrUseLastResponse
	}

	for _, path := range []string{"/index.html"} {
		t.Run(path, func(t *testing.T) {
			resp, err := client.Get(srv.URL + path)
			if err != nil {
				t.Fatalf("GET %s: %v", path, err)
			}
			defer resp.Body.Close()
			if resp.StatusCode != http.StatusOK {
				t.Errorf("status = %d, want 200 (Location header: %q)",
					resp.StatusCode, resp.Header.Get("Location"))
			}
			if loc := resp.Header.Get("Location"); loc != "" {
				t.Errorf("Location = %q, want empty (no redirect)", loc)
			}
		})
	}
}

// TestResolveStaticPath checks the URL → relative-path mapping rules in
// isolation, independent of the filesystem.
func TestResolveStaticPath(t *testing.T) {
	cases := []struct {
		in   string
		want string
		ok   bool
	}{
		{"/", "index.html", true},
		{"/foo.html", "foo.html", true},
		{"/nested/foo.html", "nested/foo.html", true},
		{"/css/chrome.css", "css/chrome.css", true},
		{"foo.html", "foo.html", true},         // leading slash is added
		{"/foo/../bar.html", "bar.html", true}, // path.Clean folds ..
		{"/\x00evil", "", false},               // null byte rejected
	}
	for _, c := range cases {
		got, ok := resolveStaticPath(c.in)
		if ok != c.ok || got != c.want {
			t.Errorf("resolveStaticPath(%q) = (%q, %v), want (%q, %v)",
				c.in, got, ok, c.want, c.ok)
		}
	}
}
