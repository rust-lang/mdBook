# CLI fixture

This fixture exercises the M4 CLI commands: `create`, `clean`, and the
`--open` flag on `build`. It is intentionally minimal so each command
can be smoke-tested in isolation without depending on a Rust toolchain or
a long-running build.

## Layout

```
cli/
├── README.md          # this file
├── book.toml          # Rust-side config (harness); kept in sync with doclens.yaml
├── doclens.yaml       # minimal valid book config (used by clean/build)
├── docs/
│   └── intro.md       # chapter file; the table of contents lives in doclens.yaml [chapters]
└── expected/
    ├── create/        # files that `create --dir /tmp/x` should produce
    └── clean-stats/   # the literal output line of `clean` on a known tree
```

## How to use

Each step is independent and can be run from the repository root.

### create

```bash
rm -rf /tmp/mdbook-cli-create
go run ./cmd/doclens create --dir /tmp/mdbook-cli-create --theme
diff -r /tmp/mdbook-cli-create tests/cli/expected/create
```

Expected files: `doclens.yaml` (with `root: docs`, `build-dir: .doclens`
and a `[chapters]` skeleton — no `SUMMARY.md`), `docs/intro.md`,
`docs/chapter_1.md`, plus the embedded theme under `theme/`. No
`.gitignore` is generated.

### clean

```bash
mkdir -p /tmp/mdbook-cli-clean/book
echo hi > /tmp/mdbook-cli-clean/book/index.html
go run ./cmd/doclens clean --dir /tmp/mdbook-cli-clean
# → "Removed 1 file, 3B total"
ls /tmp/mdbook-cli-clean/book  # should not exist
```

### clean --dest-dir

```bash
mkdir -p /tmp/mdbook-cli-custom/sub
echo hi > /tmp/mdbook-cli-custom/sub/x.html
go run ./cmd/doclens clean --dir . --dest-dir /tmp/mdbook-cli-custom
# → removes the custom tree without loading the book
```

### build --open

```bash
# --open only has a visible effect on a desktop session; in CI it should
# still return 0 once the book is written.
go run ./cmd/doclens build --dir tests/cli --open
ls tests/cli/book/index.html  # written before the open attempt
```

## Notes

* The fixture is deliberately separate from `basic/` and `nested/` so
  CLI regressions can be investigated without touching the rendering
  golden files.
* Acceptance (M4.9) is deferred until the `build` memory regression
  in basic/nested is resolved; this fixture exists for the post-fix
  re-run.
