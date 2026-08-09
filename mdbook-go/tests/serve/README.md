# Serve fixture

This fixture exercises the M5 watch/serve commands. It is intentionally
minimal: one book chapter plus an `examples/` directory registered via
`[build] extra-watch-dirs`. The point is to verify that:

1. `mdbook watch` picks up changes to `src/` and `examples/`.
2. `mdbook serve` serves the rendered book over HTTP and triggers a
   rebuild when files change.
3. The `__livereload` WebSocket endpoint emits "reload" after each
   rebuild.

## Layout

```
serve/
├── README.md
├── book.toml          # Rust-side config (harness); kept in sync with doclens.yaml
├── doclens.yaml
├── src/
│   ├── SUMMARY.md      # 仅供 harness 的 Rust 参考腿；Go 侧读 doclens.yaml [chapters]
│   └── intro.md
└── examples/
    └── snippet.md
```

`doclens.yaml` declares `examples/` as an extra-watch-dir so a change
inside that directory (which the source scanner does not pick up) still
triggers a rebuild — this exercises M5.4 alongside the basic M5.1
behaviour.

## How to use

Each command should be run from the repository root.

### watch (poll)

```bash
go run ./cmd/doclens watch --dir fixtures/serve --watcher poll
# in another shell:
echo "extra line" >> fixtures/serve/src/intro.md
# → "Files changed: ..." in the watcher output
```

### watch (native)

```bash
go run ./cmd/doclens watch --dir fixtures/serve --watcher native
```

### serve

```bash
go run ./cmd/doclens serve --dir fixtures/serve --port 3000
# open http://localhost:3000 in a browser
# the browser console should show a websocket connection
# to /__livereload and a "reload" message after each rebuild
```

### serve --open

```bash
go run ./cmd/doclens serve --dir fixtures/serve --open
# equivalent to invoking `Open(serving_url)` after the listener is up
```

## Notes

* The fixture is separate from `cli/` so watch regressions can be
  investigated without touching the CLI smoke tests.
* Acceptance (M5.10) is deferred until the `build` memory regression
  on `basic` is fixed; this fixture exists for the post-fix re-run.
* The `__livereload` endpoint constant is the same in
  `internal/serve/serve.go` and the Rust CLI
  (`src/cmd/serve.rs::LIVE_RELOAD_ENDPOINT`); the bundled book.js
  connects there directly.
