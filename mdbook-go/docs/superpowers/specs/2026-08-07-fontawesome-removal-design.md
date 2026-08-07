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
- We are **not** keeping `<template id="fa-X">` containers or `book.js`'s
  `getElementById('fa-X').innerHTML` lookups. Both were deleted outright
  (Option B, see §7 resolution note). The rendered page carries no
  `fa-svg` icons, no `<i class="fa-…">` rewrites, no template containers,
  and no JS hook reading them. Buttons that used to be wired via those
  templates render with empty bodies and rely on their `title`/`aria-label`
  attributes alone for affordance.

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

### 3.3 Templates: drop `{{fa ...}}` calls AND `<template id="fa-X">` containers

`book.js` used to call `document.getElementById('fa-eye')` /
`'fa-eye-slash'` / `'fa-copy'` / `'fa-play'` / `'fa-clock-rotate-left'` and
assign `.innerHTML` to dynamically-injected buttons (block collapse/expand
toggle, code copy button, playground run button, history reset button). Both
the 5 `<template id="fa-X">` containers and the matching 5 `book.js` lookups
were deleted outright (Option B, see §7 resolution note). The page now carries
zero icon-system artifacts downstream of `{{fa ...}}`.

Three templates carried this pattern; all three get the same surgery:

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
- 5 `<template id="fa-X">…</template>` blocks → **deleted**. The matching
  `getElementById('fa-X').innerHTML` lines in `theme/js/book.js` were deleted
  in the same change, so no orphan JS lookup remains.
  (Originally the spec called for `id` retention so JS lookups would still
  return empty strings; that compromise was rejected in favour of outright
  deletion — see §7 resolution note.)

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
| `README.md:36,59` | drop the `fontawesome/` row from the directory-tree bullet list and the reference to `internal/fontawesome` in the test-coverage bullet |
| `MIGRATION.md:141-149` | drop the "Font Awesome" subsection in its entirety (it's the contract this spec fulfils) |

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
                            page HTML ─▶ no icon-system artifacts reach the page
```

There is no helper, no lookup, no SVG payload, and **no** `<template
id="fa-X">` containers anywhere in the output either — they were deleted
outright (Option B), so no icon-system symbol reaches the rendered HTML.
Output `<main class="markdown-body">` content is identical to the previous
output *minus* the SVG bodies in chrome; the surrounding chrome structure
(button positions, theme toggle list, search-bar slot, prev/next links) is
unchanged.

## 5. Failure modes

| Failure | Behaviour |
|---------|-----------|
| Markdown contains `<i class="fa-bars"></i>` (the path `convertFontAwesome` used to rewrite) | goldmark emits the `<i>` raw — the user gets an empty inline box. Acceptable: this was never a documented public API; the deprecation notice already steered users to `<img>`/theme-embed. |
| Markdown contains `{{fa "..." "..."}}` (Handlebars-style; would-be-handled by the helper) | Not applicable — mdbook-go is not a Handlebars engine. The text reaches the page literal. No regression vs. today. |
| User-built theme templates reference `{{fa "..." "..."}}` | The template helper no longer exists. `html/template` will execute the call site as a no-op (it's just a missing-FuncMap name), the call returns the empty string, and the surrounding HTML renders around it. Same effect as deleting the helper. |
| `book.js` still calls `getElementById('fa-X').innerHTML` | Not applicable — those calls were deleted in the same change (Option B). If a *future* patch reintroduces them, `<template id="fa-X">` would resolve to `null` and the `.innerHTML =` assignment would throw a `TypeError`; the surrounding button-creation code path would have to be guarded or the matching containers re-added. |

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
   - **Zero** `<template id="fa-X">` blocks anywhere in the page
     (deletions per Option B, see §7 resolution note).
5. **Rust testsuite regression.** Per the existing 8-gap block-list referenced
   in memory `[[mdbook-go-deleting-rust-blockers]]` (A5 entries, §"C 类"), the
   fontawesome fixture is **not** in scope for this spec — the larger
   Rust-testsuite harness is still blocked by unrelated A/B gaps. We are
   removing the production wiring here; the rust-side fixture can be unskipped
   later in a separate workstream.

## 7. Acceptance criteria

> **Resolution note (2026-08-07):** the original criterion also grepped for the
> literal `"fa-`, which contradicted §2/§3.3's original instruction to *keep*
> the `<template id="fa-X">` containers. The maintainer resolved this in
> favour of **Option B — deletion**: the 5 `<template id="fa-X">` elements and
> the 5 matching `getElementById('fa-X')` lookups in `theme/js/book.js` are
> gone. The `"fa-` literal is therefore dropped from the grep below.
>
> §2, §3.3, §4, §5, and §6 have all been rewritten to describe the actual
> final state (templates + book.js lookups both deleted outright), so readers
> reaching this spec today will not be misled by the original "retain the
> ids" wording.

- `git grep -n 'fontawesome\|font-awesome\|fa-svg'` over `cmd`, `internal`,
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
- Re-introducing the hbs engine as the production renderer. The hbs engine is
  off the production hot path (see comment in `internal/render/render.go:1-9`).
  We still touch `theme/templates/index.hbs` and `fixtures/cli/expected/init/theme/index.hbs`
  because the hbs engine's byte-level regression test depends on them, but
  this is not "promoting hbs back" — it is keeping the regression fixture
  honest.
- Refreshing the affected golden files beyond what the regeneration
  produces. Any extra cleanup of those is a separate concern.
