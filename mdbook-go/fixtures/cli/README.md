# CLI fixture

This fixture exercises the M4 CLI commands: `init`, `clean`, `test`, and
the `--open` flag on `build`. It is intentionally minimal so each command
can be smoke-tested in isolation without depending on a Rust toolchain or
a long-running build.

## Layout

```
cli/
├── README.md          # this file
├── book.toml          # minimal valid book config (used by test/clean/build)
├── src/
│   ├── SUMMARY.md
│   └── intro.md       # contains one fenced ```rust block for `test`
└── expected/
    ├── init/          # files that `init -dir /tmp/x` should produce
    └── clean-stats/   # the literal output line of `clean` on a known tree
```

## How to use

Each step is independent and can be run from the repository root.

### init

```bash
rm -rf /tmp/mdbook-cli-init
go run ./cmd/mdbook init -dir /tmp/mdbook-cli-init -theme
diff -r /tmp/mdbook-cli-init fixtures/cli/expected/init
```

Expected files: `book.toml`, `.gitignore`, `src/SUMMARY.md`,
`src/intro.md`, `src/chapter_1.md`, plus the embedded theme under
`theme/`.

### clean

```bash
mkdir -p /tmp/mdbook-cli-clean/book
echo hi > /tmp/mdbook-cli-clean/book/index.html
go run ./cmd/mdbook clean -dir /tmp/mdbook-cli-clean
# → "Removed 1 file, 3B total"
ls /tmp/mdbook-cli-clean/book  # should not exist
```

### clean --dest-dir

```bash
mkdir -p /tmp/mdbook-cli-custom/sub
echo hi > /tmp/mdbook-cli-custom/sub/x.html
go run ./cmd/mdbook clean -dir . -dest-dir /tmp/mdbook-cli-custom
# → removes the custom tree without loading the book
```

### test

```bash
# Requires rustdoc on PATH.
go run ./cmd/mdbook test -dir fixtures/cli
go run ./cmd/mdbook test -dir fixtures/cli -chapter "Intro"
```

When rustdoc is missing, the command should exit non-zero with a
"`rustdoc` not found" message.

### build --open

```bash
# -open only has a visible effect on a desktop session; in CI it should
# still return 0 once the book is written.
go run ./cmd/mdbook build -dir fixtures/cli -open
ls fixtures/cli/book/index.html  # written before the open attempt
```

## Notes

* The fixture is deliberately separate from `basic/` and `nested/` so
  CLI regressions can be investigated without touching the rendering
  golden files.
* Acceptance (M4.9) is deferred until the `build` memory regression
  in basic/nested is resolved; this fixture exists for the post-fix
  re-run.
