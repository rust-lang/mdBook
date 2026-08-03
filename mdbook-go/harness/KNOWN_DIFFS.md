# Known Rust vs Go differences

The diff harness exits non-zero on any byte difference between the Rust
and Go outputs. Every line below is **expected** and corresponds to either
an intentionally skipped fixture or a parser-level deviation that requires
non-trivial work to close.

Last updated: 2026-08-03 — both `basic` (40 files) and `nested` (48 files)
produce byte-identical output, so no fixture is currently in `SKIP`.

## Skipped fixtures

(None. Both `basic` and `nested` pass strict diff.)

## Markdown parser deviations (goldmark vs pulldown-cmark)

These are tracked in `internal/html/markdown_golden_test.go`'s
`knownDeviations` slice. The corresponding fixtures under
`tests/testsuite/markdown/` are skipped from the golden regression. They
do **not** affect `basic` or `nested`; they only show up when reusing the
Rust `testsuite` fixtures as Go goldens, which is a separate test path.

1. `tests/testsuite/markdown/definition_lists/definition_lists.md` —
   goldmark requires a single-line plain-text term; inline links or
   multi-line terms do not become `<dt>`.
2. `tests/testsuite/markdown/basic_markdown/html.md` — when an opening
   HTML tag spans two lines, goldmark treats it as a block element while
   pulldown-cmark falls back to inline HTML inside a paragraph.

Fixing either requires swapping out part of goldmark's block parser;
deferred until a fixture explicitly demands it.

## Items removed from this file

These were listed under M1/M2 and have since been closed:

- `404.html`, `print.html`, `toc.html`, `toc.js`, `searchindex.js` — M2.
- Font Awesome CSS/JS/icons and the menu bar / sidebar / footer JS — M2.
- Theme asset hashing and `{{ resource }}` rewriting — M2.
- `redirect` table support — M2.
- `additional-css` and `fold` rendering — M2 (nested fixture).
- Strict-mode byte-for-byte equivalence on `basic` and `nested` — M2.