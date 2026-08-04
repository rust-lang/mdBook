# Migrating from mdBook (Rust) to mdbook-go

`mdbook-go` is a Go reimplementation of [mdBook][upstream] living in this
repository as `mdbook-go/`. The Rust source tree (`src/` + `crates/`)
remains the canonical baseline; mdbook-go is a parallel, milestone-by-
milestone port. This document describes what works, what's different,
and how to fall back when the Go build misbehaves.

> The authoritative progress checklist is `../doc/plan/progress.md`.
> As of 2026-08-04 every M1–M5 item is closed; only M6 release work
> (CI, cross-build, license report, benchmarks) and the frozen M3
> external-plugin path remain.

[upstream]: https://github.com/rust-lang/mdBook

## Status snapshot

| Milestone | Description | State |
|---|---|---|
| M1 | Core loader + minimal `build` | ✅ |
| M2 | HTML renderer (goldmark → mdBook DOM) | ✅ — byte-identical to Rust on `basic` (40 files) and `nested` (48 files) |
| M3 | preprocessor / renderer protocol + built-in `links` / `index` | ✅ code, **frozen** for external plugin e2e (see below) |
| M4 | CLI surface: `init` / `build` / `clean` / `test` / `watch` / `serve` / `completions` + unified error handler | ✅ — byte-identical to Rust on `cli` (37 files) |
| M5 | `watch` (poll + native) / `serve` + WebSocket live reload | ✅ — byte-identical to Rust on `serve` (38 files) |
| M6 | cross-platform builds / CI / license report / docs / release | 🚧 (this milestone) |

The harness that proves the byte-equivalence is `mdbook-go/harness/diff.sh`;
see "Running the diff harness" below.

## Installation

### Pre-compiled binaries

Once the M6.7 release process is in place, pre-built binaries land on the
GitHub Releases page alongside the existing Rust ones. The archive
naming follows the Rust convention:

```text
mdbook-go-<version>-<goos>-<goarch>.tar.gz   # linux + darwin
mdbook-go-<version>-<goos>-<goarch>.zip      # windows
```

For now the easiest path is to build from source:

```bash
git clone https://github.com/rust-lang/mdBook.git
cd mdBook/mdbook-go
go build -o ./bin/mdbook-go ./cmd/mdbook
# optional: copy to a directory on your PATH
install -m 0755 ./bin/mdbook-go /usr/local/bin/mdbook-go
```

Requires Go 1.26 or newer.

### Building from source

```bash
cd mdbook-go
go build -o bin/mdbook-go ./cmd/mdbook
```

The resulting binary is statically linked and ~14 MB. For a smaller
binary, set `GO_LDFLAGS="-s -w"` before `go build`.

## CLI parity

Every mdBook subcommand is implemented. Flags match the Rust `--long`
names; short flags are added where Rust ships them.

| Subcommand | Rust flags | Go flags | Notes |
|---|---|---|---|
| `init` | `[--dir]` `--theme` `--force` `--title <T>` `--ignore <git\|none>` | `-dir` `-theme` `-force` `-title` `-ignore` | `--force` is accepted but currently a no-op (no interactive prompts in the Go port). |
| `build` | `[--dir]` `[--dest-dir DIR]` `[--open]` | `-dir` `-dest-dir` `-open` | `-open` calls `open` / `xdg-open` / `cmd /c start`. |
| `clean` | `[--dir]` `[--dest-dir DIR]` | `-dir` `-dest-dir` | Output line matches Rust `Clean::Display` exactly (tested on `cli` fixture). |
| `test` | `[--dir]` `[--chapter NAME]` `[--library-path DIR[,DIR...]]` | `-dir` `-chapter` `-library-path` | Spawns `rustdoc <chapter> --test` per non-draft chapter, mirrors Rust stderr. |
| `watch` | `[--dir]` `[--dest-dir DIR]` `[--open]` `[--watcher poll\|native]` | `-dir` `-dest-dir` `-open` `-watcher` | Default watcher is `native` (Rust's default is `poll` — see "Differences"). |
| `serve` | `[--dir]` `[--dest-dir DIR]` `[--hostname H]` `[--port P]` `[--open]` | `-dir` `-dest-dir` `-hostname` `-port` `-open` | `net/http` + gorilla/websocket on `__livereload`. |
| `completions <shell>` | (via `clap_complete`) | `-shell <bash\|zsh\|fish\|powershell>` or positional | See "Shell completions" below. |

Top-level flags in Rust mdBook that are **not** yet implemented in Go:

- `--help` / `-h` per subcommand — Go's stdlib `flag` package already
  prints a usage line on `-h` / `-help`, but the formatted `after_help`
  epilog Rust ships is not replicated.

## Configuration compatibility

`book.toml` parses identically to Rust. The `[book]`, `[build]`, `[output.*]`,
and `[preprocessor.*]` tables work; dynamic fields (`output.html.theme`,
`output.html.playground.edit-url-path`, etc.) round-trip through the
preprocessor protocol wire format described in `internal/plugin/protocol.go`.

Environment variable overrides (`MDBOOK_BOOK__TITLE`, `MDBOOK_BUILD__BUILD_DIR`,
…) follow Rust's `MDBOOK_<TABLE>__<KEY>` convention with double-underscore
as the section separator.

## Plugin compatibility

The preprocessor / renderer **wire protocol** (the JSON written to a
plugin's stdin and read from its stdout) is wire-compatible with Rust
mdBook. A plugin written for Rust mdBook will receive the same JSON,
parse it the same way, and the Go host will consume its output the same
way Rust does. See `internal/plugin/protocol.go` for the field mapping.

Built-in preprocessors in scope:

- `links` — `{{#include}}`, `{{#rustdoc_include}}`, `{{#playground}}`,
  `{{#title}}`, `\{{#…}}` escape. Line ranges and anchor syntax
  match Rust; nested includes respect `max-link-nested-depth = 10`.
- `index` — `README.md` → `index.md` rewrite, case-insensitive.

External preprocessors / renderers: **frozen at the code level but not
end-to-end validated**. The fixtures (`fixtures/external-plugin/`) and
the harness SKIP entry exist; picking this back up is a matter of
re-enabling `external-plugin` in `harness/diff.sh` once a real
third-party plugin needs to be supported.

## Markdown / HTML differences

`mdbook-go` parses Markdown with [goldmark][goldmark]; Rust mdBook uses
[pulldown-cmark][pulldown-cmark]. For the common Markdown subset the
two produce identical HTML, but three corner cases differ:

[goldmark]: https://github.com/yuin/goldmark
[pulldown-cmark]: https://github.com/rust-lang/pulldown-cmark

1. **Definition lists** — goldmark requires a single-line term;
   multi-line terms or terms containing inline formatting are not
   wrapped in `<dt>`. Affected: `tests/testsuite/markdown/definition_lists`.
2. **Custom header attributes** — goldmark ignores Rust's
   `{#id .class}` syntax; the raw text leaks into the heading. Affected:
   `tests/testsuite/markdown/custom_header_attributes`.
3. **HTML block boundary** — when an opening tag spans two lines,
   goldmark treats the content as an HTML block, pulldown-cmark
   keeps it inline. Affected: `tests/testsuite/markdown/basic_markdown`.

Each difference is registered in `internal/html/markdown_golden_test.go`
under `knownDeviations`. None affect the `basic` / `nested` /
`cli` / `serve` fixtures used for byte-equivalence validation.

4. **Font Awesome** — *deprecated*. The Go port embeds only 15 of the
   icons in Font Awesome Free 6.2.0; growing the table to full parity
   costs ~700 KB of binary size for a feature the upstream maintainers
   are phasing out. The `{{fa ...}}` handlebars helper continues to
   work for the embedded icons but emits a one-shot stderr warning on
   first use, pointing users at the package doc comment
   (`internal/fontawesome/fontawesome.go`). Affected fixture:
   `tests/testsuite/rendering/fontawesome`. Migrate by embedding the
   SVG directly in your theme or via `<img>`.

## Shell completions

Generate a script for your shell and source it once:

```bash
# bash
mdbook-go completions bash > ~/.local/share/bash-completion/completions/mdbook-go

# zsh (drop into any directory on $fpath)
mdbook-go completions zsh > "${fpath[1]}/_mdbook-go"

# fish
mdbook-go completions fish > ~/.config/fish/completions/mdbook-go.fish

# PowerShell — eval from your $PROFILE
mdbook-go completions powershell | Out-String | Invoke-Expression
```

`completions` also accepts `--shell <name>` instead of a positional
argument; unknown shell names produce a Rust-style "Caused by:"
error and exit 101.

## Error handling

Errors are printed to stderr and the process exits with code 101,
matching Rust mdBook's `utils::log_backtrace` + `std::process::exit(101)`.
Multi-layer errors (e.g. `Load → read book.toml → open file`) print
each layer on its own line:

```text
read /tmp/foo/book.toml: open /tmp/foo/book.toml: no such file or directory
	Caused by: open /tmp/foo/book.toml: no such file or directory
	Caused by: no such file or directory
```

The subcommand prefix Rust's earlier Go port used (`init: …`, `build: …`)
is dropped to match the upstream format.

## Running the diff harness

The `mdbook-go/harness/diff.sh` script is the canonical compatibility
check. It runs both `cargo run --bin mdbook build` and `./bin/mdbook-go
build` on the same fixture, then `diff -r`s the outputs. Any byte
difference fails the script (strict mode). Fixtures with known deferred
differences are listed in the `SKIP` array at the top of the script.

```bash
cd mdbook-go
MDBOOK_RUST_BIN=$(pwd)/../target/debug/mdbook ./harness/diff.sh
# → OK   basic (40 files identical)
# → OK   nested (48 files identical)
# → OK   cli (37 files identical)
# → OK   serve (38 files identical)
# → SKIP external-plugin (M3 frozen)
```

There is also `harness/diff_rust_testsuite.sh`, which walks the Rust
project's own `tests/testsuite/` directory and runs both implementations
on every buildable fixture there. As of the last run: **22 PASS / 7 DIFF
/ 17 SKIP|BUILD_FAIL**. The 7 DIFFs are all in the "Markdown / HTML
differences" section above.

## Fallback

If a regression slips through, fall back to the Rust binary without
uninstalling mdbook-go:

1. **Temporarily rename** `mdbook-go` on your `PATH` so `mdbook` resolves
   to the Rust binary:

   ```bash
   sudo mv /usr/local/bin/mdbook-go /usr/local/bin/mdbook-go.disabled
   ```

2. **Or set up an alias** that points `mdbook` at the Rust build:

   ```bash
   alias mdbook=/path/to/mdBook/target/debug/mdbook
   ```

3. **For CI**, drop the Go binary entirely and rely on the pre-installed
   Rust `mdbook` — your existing `cargo install mdbook` workflow keeps
   working unchanged.

To permanently revert, delete the `mdbook-go/` directory and remove the
binary from your `PATH`. The Rust implementation is the canonical
baseline and is never modified by the Go port.

## Reporting regressions

Open an issue against the mdBook repository with:

1. The fixture (`book.toml`, `src/SUMMARY.md`, plus the affected
   `*.md` files).
2. The output of `./bin/mdbook-go --help` / `version`.
3. The exact error message, including the `Caused by:` chain.
4. The output of `./harness/diff.sh <fixture>` showing the divergence.

The harness's strict mode makes step 4 sufficient on its own for most
regressions.