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

## Layout

```text
mdbook-go/
├── cmd/mdbook/        CLI entry point
├── internal/
│   ├── book/          Book / Chapter model
│   ├── config/        book.toml loading + env override
│   ├── summary/       SUMMARY.md parser
│   ├── markdown/      goldmark wrapper
│   ├── driver/        MDBook, loader, build, init
│   ├── html/          reserved for M2 renderer
│   ├── plugin/        reserved for M3 preprocessor / renderer protocol
│   └── watch/         reserved for M5 watcher
├── fixtures/          shared test books
├── harness/           Rust-vs-Go diff harness
└── go.mod
```

## Current milestone

**M1: core loader + minimum build.**

Implemented:

- `Book`, `Chapter`, `SectionNumber` data model.
- `book.toml` parsing with dynamic `output.*` and `preprocessor.*` sections.
- `MDBOOK_*` environment variable override.
- `SUMMARY.md` parser supporting part titles, separators, drafts and
  nested links.
- Disk loader (UTF-8 BOM stripping, chapter tree assembly).
- Markdown → HTML using goldmark.
- Minimal HTML output (one file per chapter + `index.html`).
- `init` and `build` subcommands.

Not yet implemented (scheduled for later milestones):

- M2: theme, TOC, search, static resources, redirects, 404, print.
- M3: preprocessor / renderer plugin protocol.
- M4: `test`, `clean`, `completions` subcommands.
- M5: `watch`, `serve`, live reload.
- M6: regression matrix, cross-platform builds, performance benchmarks.

## Harness

```bash
./harness/diff.sh basic
```

The harness builds both the Rust and Go binaries, runs `mdbook build` on
the same fixture into separate output directories, then diffs the result.

The diff at M1 is expected to be large: the Rust binary emits a fully themed
page with a sidebar, JavaScript runtime, `404.html`, `print.html` and
`toc.html`, while the Go binary emits a minimal HTML envelope. See
`harness/KNOWN_DIFFS.md` for the documented differences.

After M2 the harness is expected to converge on the chapter body; M3 onward
should align the plugin JSON protocol.

## Running the Rust test suite

The existing Rust test suite (`cargo test --workspace`) still exercises the
Rust implementation. The Go port is verified by running both binaries on
the same fixture and diffing the output. This is documented in
`doc/plan/README.md` under "对照 harness".
