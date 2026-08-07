# mdbook-go

Go port of [mdBook](https://github.com/rust-lang/mdBook). This directory is a
parallel implementation: the Rust source tree under `src/` and `crates/`
stays as the canonical baseline; everything under `mdbook-go/` is a
reimplementation in Go that aims to reach behavioural and output
compatibility milestone by milestone.

See `../doc/plan/README.md` for the full plan and `../doc/plan/progress.md`
for live status.

## Quick start

```bash
cd mdbook-go
go build -o bin/mdbook-go ./cmd/mdbook
./bin/mdbook-go build -dir fixtures/basic -dest-dir /tmp/out
```

`./bin/mdbook-go init [-dir DIR] [-theme]` creates a new book skeleton
(`book.toml`, `SUMMARY.md`, first chapter, `.gitignore`).

## Layout

```text
mdbook-go/
├── cmd/mdbook/        CLI entry point (build, init, version)
├── internal/
│   ├── book/          Book / Chapter / SectionNumber model
│   ├── config/        book.toml loading + env override + html config
│   ├── summary/       SUMMARY.md parser (nested, draft, part title, separator)
│   ├── driver/        MDBook orchestrator, loader, build, init
│   ├── utils/         HTML escape, path_to_root, slug, dedup id, file copy
│   ├── theme/         in-tree + on-disk theme resolution
│   ├── hbs/           Handlebars subset engine (standalone whitespace, helpers)
│   ├── html/          goldmark → mdBook node tree → HTML serialization
│   ├── static/        static asset hashing + {{ resource }} rewrite
│   ├── search/        elasticlunr-compatible searchindex.js
│   ├── render/        HTML backend main loop (toc, print, 404, redirect)
│   ├── plugin/        reserved for M3 preprocessor / renderer protocol
│   └── watch/         reserved for M5 watcher
├── fixtures/          shared test books (basic, nested)
├── harness/           Rust-vs-Go diff harness (strict mode)
└── theme/             go:embed default frontend assets
```

## Current milestone

**M2: HTML renderer — closed. Strict-mode harness passes on basic + nested.**

Verified:

- `fixtures/basic` — 40 files byte-identical to the Rust build.
- `fixtures/nested` — 48 files byte-identical to the Rust build
  (multi-level nesting, tables, footnotes, admonitions, task lists,
  `additional-css`, `fold`, draft chapters, part titles, separators,
  prefix / numbered / suffix zones, redirects).
- `internal/hbs`, `internal/search`, `internal/html` golden tests pass
  against Rust output / Rust fixtures.

Implemented:

- `Book`, `Chapter`, `SectionNumber` data model with hierarchical
  numbering and per-chapter subdirectory preservation.
- `book.toml` parsing with dynamic `output.*` and `preprocessor.*`
  sections; `MDBOOK_*` environment variable override.
- `SUMMARY.md` parser: arbitrary nesting, prefix / numbered / suffix
  zones, part title, separator, draft, subdirectory chapters.
- Disk loader with UTF-8 BOM stripping.
- `goldmark` driven Markdown → mdBook node tree → HTML, with extensions
  for tables, footnotes, task lists, strikethrough, definition lists,
  admonitions, smart punctuation, math, playground, hide-lines,
  font awesome.
- Title IDs and dedup; `.md` → `.html` link rewriting.
- `index.html`, per-chapter pages, `toc.html`, `toc.js`,
  `404.html`, redirects, `.nojekyll`.
- Static asset collection, SHA-256 fingerprinting, `{{ resource }}`
  rewriting (CSS/JS/font/icons).
- Inlined default theme via `go:embed`; user `theme/` overrides file by
  file.
- `elasticlunr`-compatible `searchindex.js` (Porter stemmer + stop
  words) — landed early because the chapter `<head>` references its
  hashed name.
- `init` and `build` subcommands.

Not yet implemented:

- M3: preprocessor / renderer plugin protocol
- M4: `test`, `clean`, `completions` subcommands
- M5: `watch`, `serve`, live reload
- M6: regression matrix, cross-platform builds, performance benchmarks

## Harness

```bash
./harness/diff.sh [fixture ...]
```

The harness builds both the Rust and Go binaries, runs `mdbook build` on
the same fixture into separate output directories, then `diff -r`s the
result.

Since M2 the comparison is strict: any difference is a failure. The only
known allowed deviations live in `harness/KNOWN_DIFFS.md` and currently
cover only two goldmark vs pulldown-cmark parser quirks — neither of
which is exercised by `basic` or `nested`.

```bash
# Override the Rust binary location if needed:
MDBOOK_RUST_BIN=/path/to/mdbook ./harness/diff.sh basic nested
```

## Running the Rust test suite

```bash
cargo test --workspace
```

The Go side reuses the Rust fixtures under `tests/testsuite/` as goldens
for its own regressions; the Rust suite itself is unchanged.