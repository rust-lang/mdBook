# Font Awesome hard removal — design

- **Date:** 2026-08-07
- **Owner:** mdbook-go maintainers
- **Status:** design, awaiting user review
- **Supersedes:** MIGRATION.md §"Font Awesome helper" (the deprecation contract
  that announced this removal)
- **Replaces plan:** ad-hoc brainstorm — no prior design artifact

## 1. Goal

Delete the `internal/fontawesome` package and every code path that consumes it.
**No replacement.** If icons are needed later (link buttons, copy affordances,
spinners), they will be added back as discrete, opt-in additions — not as a
catch-all upstream-port feature.

## 2. Non-goals

- We are **not** designing a generic icon system. No plugin point, no helper API,
  no theme-level icon registry. Anyone who later needs an icon embeds the SVG
  directly in their theme, exactly as the now-deleted deprecation notice
  already advised.
- We are **not** preserving the byte-exact SVG output for any markdown that
  happens to contain `<i class="fa-X">`. The replaceable form was a tiny niche
  feature; the markdown source will now pass through to the output unchanged
  (goldmark leaves raw `<i>` elements alone).
- We are **not** deleting `book.js` or its use of `<template id="fa-X">`
  elements. Those `<template>` containers stay in the page with empty bodies so
  the JS `innerHTML` reads work against the same ids they read against today.

## 3. Scope of deletion

### 3.1 Source files removed

| File | Reason |
|------|--------|
| `internal/fontawesome/fontawesome.go` | public package API (`Type`, `TypeFromString`, `SVG`, `Span`) |
| `internal/fontawesome/icons.go` | the 14 inline `SolidBars`…`BrandsGithub` constants plus the `iconsSolid`/`iconsRegular`/`iconsBrands` maps |
| `internal/fontawesome/fontawesome_test.go` | byte-exact Rust equivalence + deprecation-warning tests; both lose purpose once the package is gone |

### 3.2 Production-callers trimmed

| File | Change |
|------|--------|
| `internal/render/render.go:21` | drop the `mdbook-go/internal/fontawesome` import |
| `internal/render/render.go:195` | drop `r.RegisterFunc("fa", faHelper)` |
| `internal/render/render.go:202-222` | drop `faHelper` and its doc block |
| `internal/tplgotpl/helpers.go:10` | drop the `mdbook-go/internal/fontawesome` import |
| `internal/tplgotpl/helpers.go:74-91` | drop the `Env.FA` method |
| `internal/html/passes.go:6` | drop the `mdbook-go/internal/fontawesome` import |
| `internal/html/passes.go:143-185` | drop `convertFontAwesome` (the `<i class="fa-X">`-rewrite pass) |
| `internal/html/builder.go:107` | drop the `b.convertFontAwesome()` call site |

### 3.3 Templates: replace `{{fa ...}}` with empty output, keep `<template id="fa-X">` containers

`book.js` calls `document.getElementById('fa-eye')` / `'fa-eye-slash'` /
`'fa-copy'` / `'fa-play'` / `'fa-clock-rotate-left'` and assigns `.innerHTML`
to dynamically-injected buttons (block collapse/expand toggle, code copy button,
playground run button, history reset button). The `id` attributes must survive.
We strip just the inner markup.

Three templates carry this pattern; all three get the same surgery:

- `internal/tplgotpl/prod/index.gohtml` (production renderer)
- `theme/templates/index.hbs` (hbs engine source — `index.gohtml` was
  translated 1:1 from this)
- `fixtures/cli/expected/init/theme/index.hbs` (what `mdbook init` writes into
  a new user's project)

In each template:

- 18 inline `{{fa ...}}` calls inside menu-bar / sidebar-toggle / theme-toggle /
  search-toggle / print-link / git-edit-link / spinner / prev-next arrows →
  empty output. Surrounding `<label>` / `<button>` / `<a>` elements keep their
  `title=` and `aria-label=` so screen readers and tooltips still describe the
  action.
- 5 `<template id="fa-X">{{fa ...}}</template>` blocks → `<template id="fa-X"></template>`.
  id retained, body zeroed.

### 3.4 CSS: drop the dead `.fa-svg*` selectors

The CSS in `theme/css/chrome.css` and `theme/css/general.css` references
`.fa-svg`/`fa-svg svg`/`.fa-svg:hover` only to style the now-removed SVGs.
After deletion these selectors match nothing, producing dead code with no
behavioural effect but ~10–30 lines of confusion.

Both files get the selectors removed. The hosted `book/css/*.css` files are
regenerated fixtures (they're copies of `theme/css/*.css` post-hash, so they
fall out of the regeneration pass in §3.6).

### 3.5 Documentation

| File | Change |
|------|--------|
| `README.md:34,36,59` | drop the `fontawesome/` row from the directory-tree bullet list and the reference to `internal/fontawesome` in the test-coverage bullet |
| `MIGRATION.md:140-149` | drop the "Font Awesome helper" subsection in its entirety (it's the contract this spec fulfils) |

### 3.6 Fixture regeneration

These fixture outputs include `<span class="fa-svg">` markup that no longer
exists. Each is a build-once-check-in golden. Regenerate by running
`./bin/mdbook-go build` for each fixture's `-dir`, then commit:

- `fixtures/basic/book/*.html` (7 files: `index`, `intro`, `chapter_1`,
  `chapter_2`, `section_1_1`, `print`, `404`)
- `fixtures/serve/book/*.html` (5 files)
- `fixtures/ts-toc-basic_toc/book/**.html` (>=10 files in nested dirs)
- `internal/hbs/testdata/intro.html` (hbs engine byte-level regression)

## 4. Data flow (post-removal)

```
Markdown source ─▶ goldmark ─▶ builder.buildTree
                                 │ (.Passes no longer calls convertFontAwesome)
                                 ▼
                            html.Serialize
                                 │
                                 ▼ html/template
                            registry.Render("index", data)
                                 │
                                 ▼ {{fa ...}} now expands to ""
                            page HTML ─▶ <template id="fa-X"></template> with empty body
```

There is no helper, no lookup, no SVG payload. Output `<main class="markdown-body">`
content is identical to today's output *minus* the SVG bodies in chrome; the
surrounding chrome structure (button positions, theme toggle list, search-bar
slot, prev/next links) is unchanged.

## 5. Failure modes

| Failure | Behaviour |
|---------|-----------|
| Markdown contains `<i class="fa-bars"></i>` (the path `convertFontAwesome` used to rewrite) | goldmark emits the `<i>` raw — the user gets an empty inline box. Acceptable: this was never a documented public API; the deprecation notice already steered users to `<img>`/theme-embed. |
| Markdown contains `{{fa "..." "..."}}` (Handlebars-style; would-be-handled by the helper) | Not applicable — mdbook-go is not a Handlebars engine. The text reaches the page literal. No regression vs. today. |
| User-built theme templates reference `{{fa "..." "..."}}` | The template helper no longer exists. `html/template` will execute the call site as a no-op (it's just a missing-FuncMap name), the call returns the empty string, and the surrounding HTML renders around it. Same effect as deleting the helper. |
| `book.js` still calls `getElementById('fa-X').innerHTML` | The `<template id="fa-X">` elements remain with empty bodies, so `.innerHTML === ""`. The dynamically-injected buttons render textless but functional. |

## 6. Testing strategy

1. **Compile gate.** `go build ./cmd/mdbook` must succeed with no references to
   `fontawesome` anywhere in the dep graph.
2. **Unit tests.** The 5 tests in `internal/fontawesome/fontawesome_test.go`
   are deleted alongside the package. No new tests required — the production
   code paths being removed had unit coverage but no semantic logic we need to
   preserve.
3. **Golden tests.** Three suites compare rendered HTML against check-ins:
   - `internal/hbs/hbs_golden_test.go` (hbs engine)
   - `internal/html/markdown_golden_test.go` (any conversion test that
     exercises `convertFontAwesome` would need to drop the test case if any
     fixture uses `<i class="fa-…">` — verification before commit.)
   - `internal/search/search_golden_test.go` (unlikely to interact)
4. **End-to-end smoke.** `./bin/mdbook-go build -dir ./fixtures/basic` →
   inspect generated `fixtures/basic/book/intro.html`. Assertions:
   - `<title>Introduction - Basic Fixture</title>` present.
   - `<main class="markdown-body">` with the rendered Markdown body.
   - Menu-bar buttons present with `title=` / `aria-label=` and no `<span class="fa-svg">`.
   - The 5 `<template id="fa-X"></template>` blocks present with empty bodies.
5. **Rust testsuite regression.** Per the existing 8-gap block-list in
   `doc/plan/testing.md`, fontawesome fixture is **not** in scope until the
   larger Rust-testsuite harness is unblocked. Calling that out so it isn't
   silently re-enabled.

## 7. Acceptance criteria

- `git grep -n 'fontawesome\|font-awesome\|fa-svg\|"fa-'` over `cmd`, `internal`,
  `theme`, `README.md`, `MIGRATION.md` returns zero hits.
- `fixtures/basic/book/intro.html` builds with a non-empty
  `<main class="markdown-body">` and no `<span class="fa-svg">` anywhere.
- `go build ./cmd/mdbook && go test ./...` passes.
- `theme/css/chrome.css` and `theme/css/general.css` contain no `.fa-svg`
  selectors.
- Fixtures regenerated and committed; the diff is reviewable as "icons gone,
  no other regression."

## 8. Out of scope (explicit)

- Building a generic theme-asset pipeline for user-supplied icons. If the user
  later wants icons they'll add them per-template — by editing the theme.
- Backporting this removal to the hbs production engine. The hbs engine is
  already off the production hot path per the comment in `render.go:1-9`.
- Refreshing the 18 affected golden files beyond what the regeneration
  produces. Any "extra" cleanup of those is a separate concern.
