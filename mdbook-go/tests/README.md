# mdbook-go fixtures

This directory holds the books the harness (`harness/diff.sh`) builds and
byte-compares against the Rust `mdbook` reference output. There are three
kinds of fixtures.

> 2026-08-09 起，目录树配置从 `SUMMARY.md` 迁移到 `doclens.yaml` 的
> `[chapters]` 段（`SUMMARY.md` 被淘汰）。fixture 中的 `SUMMARY.md`
> 文件**保留**，但仅供 harness 的 Rust 参考腿使用（Rust mdbook 仍读
> book.toml + SUMMARY.md）；Go 侧只读 `doclens.yaml` 的 `[chapters]`。
> 修改目录树时必须同时更新两处，保持两边一致。
>
> ⚠️ 下方 "Result" 列的 `OK (N files)` 是 2026-08-07 之前（硬删除
> print / playground / fontawesome、主题改版之前）的记录。当前 Rust
> 基线仍输出 print.html、fonts/、favicon 与 Font Awesome 图标，Go
> 侧已删除，`harness/diff.sh` 对每个 fixture 都会在主题资产上 DIFF；
> 章节正文与 `toc.js` 目录结构已对齐（2026-08-09 修复了
> `toc_render.go` 的层级 +1 偏差）。fixture 的 `book/` 输出目录是
> 当前 Go 构建的参考输出，随构建更新。

## Hand-curated (M1 + M2 + M5 era)

| Name | What it covers |
|---|---|
| `basic/` | Single-chapter book with one `rust` fenced block. The canonical M1 acceptance fixture. |
| `nested/` | Four-level nested `[chapters]` tree, sub-directory chapters, tables, footnotes, admonitions, task lists, code blocks, redirects, `additional-css`, fold, separator, part titles, prefix/suffix chapters, draft. The canonical M2 acceptance fixture. |
| `cli/` | Minimal book exercising the `init` / `clean` / `test` / `build -open` CLI paths. M4 acceptance. |
| `serve/` | Book with `extra_watch_dirs` + `additional-css`. M5 acceptance. |
| `external-plugin/` | Three Node preprocessor scripts (banner / footer / noisy). M3 acceptance, **frozen** — see `doc/plan/progress.md`. Listed in `harness/diff.sh`'s `SKIP` list. |

## Imported from `tests/testsuite/` (M6.1 era)

Each entry below is a verbatim copy of `tests/testsuite/<category>/<name>/`
with a `ts-<category>-<name>` prefix so the harness can match them with
a single glob (`tests/ts-*/`) while keeping the upstream category
visible in the directory name. Twelve of these pass byte-identical; one
(`ts-markdown-basic_markdown`) is a known goldmark / pulldown-cmark
divergence and is registered in `harness/diff.sh`'s `SKIP` list.

| Name | Origin | Result |
|---|---|---|
| `ts-build-basic_build/` | `tests/testsuite/build/basic_build/` | OK (37 files) |
| `ts-build-create_missing/` | `tests/testsuite/build/create_missing/` | OK (37 files) |
| `ts-config-empty/` | `tests/testsuite/config/empty/` | OK (37 files) — 2026-08-09 起 `doclens.yaml` 显式声明 `root: src` / `build-dir: book`（Go 默认已改为 docs/.doclens，与 Rust 默认 src/book 不同，空配置无法再两边对齐） |
| `ts-index-basic_readme/` | `tests/testsuite/index/basic_readme/` | OK (39 files) |
| `ts-markdown-admonitions/` | `tests/testsuite/markdown/admonitions/` | OK (37 files) |
| `ts-markdown-basic_markdown/` | `tests/testsuite/markdown/basic_markdown/` | SKIP (goldmark HTML block boundary; see `MIGRATION.md`) — 2026-08-09 起 `doclens.yaml` 显式声明 `root: src` / `build-dir: book` |
| `ts-playground-disabled_playground/` | `tests/testsuite/playground/disabled_playground/` | OK (37 files) |
| `ts-playground-playground_on_rust_code/` | `tests/testsuite/playground/playground_on_rust_code/` | OK (37 files) |
| `ts-print-duplicate_ids/` | `tests/testsuite/print/duplicate_ids/` | OK (38 files) |
| `ts-print-relative_links/` | `tests/testsuite/print/relative_links/` | OK (39 files) |
| `ts-redirects-redirects_are_emitted_correctly/` | `tests/testsuite/redirects/redirects_are_emitted_correctly/` | OK (40 files) |
| `ts-theme-custom_fonts_css/` | `tests/testsuite/theme/custom_fonts_css/` | OK (24 files) |
| `ts-theme-empty_theme/` | `tests/testsuite/theme/empty_theme/` | OK (with an empty `theme/index.hbs` added — Rust requires the dir to exist) |
| `ts-includes-all_includes/` | `tests/testsuite/includes/all_includes/` | OK (46 files) — recursive `{{#include}}` self-reference + `{{#include FILE:ANCHOR}}` stripping orphan ANCHOR directives (`see internal/runner/links_test.go`)；2026-08-09 起 `doclens.yaml` 显式声明 `root: src` / `build-dir: book` |
| `ts-test-passing_tests/` | `tests/testsuite/test/passing_tests/` | OK (42 files) — happy-path coverage of `{{#include}}` / `{{#playground}}` / anchor mode；2026-08-09 起 `doclens.yaml` 显式声明 `root: src` / `build-dir: book` |
| `ts-toc-basic_toc/` | `tests/testsuite/toc/basic_toc/` | OK (45 files) — exercises the loader's handling of an indented bare-link continuation entry, which the Rust parser drops (`internal/runner/loader.go`); the corresponding `chapters` entry is therefore absent from `doclens.yaml` |

The `ts-theme-empty_theme/theme/` directory was added during M6.1 import
because Rust refuses to render into a non-existent theme directory; the
upstream fixture relied on `BookTest::from_dir` creating it implicitly.

## Adding a new fixture

1. Pick the smallest Rust testsuite case that exercises the feature you
   care about. Avoid ones whose tests use `expect_failure()` or that
   reference external preprocessor commands (the M3 external-plugin
   path is frozen).
2. Copy it under `tests/ts-<category>-<name>/`.
3. Add any required scaffold (e.g. the empty `theme/index.hbs` above).
4. Run `./harness/diff.sh ts-<category>-<name>` and confirm the result is
   `OK` (or, if the divergence is known, register it in `SKIP`).
5. If the fixture needs docs (e.g. a custom command-line invocation),
   add a paragraph to this README.
