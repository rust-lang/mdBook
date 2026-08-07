# Font Awesome Hard Removal — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Delete `internal/fontawesome` and every code/template/CSS path that references it, with no replacement. Fixtures regenerate to a check-in clean diff.

**Architecture:** Four sequential surgical tasks — backend code → templates → CSS → docs+goldens. Each task ends with its own `go build` / acceptance-gate commit, so a reviewer can reject one task without bouncing the others.

**Tech Stack:** Go 1.26.4, html/template (production renderer), embed.FS for prod templates, goldmark for HTML→node-tree, byte-level golden tests for regression.

**Spec:** [2026-08-07-fontawesome-removal-design.md](../specs/2026-08-07-fontawesome-removal-design.md)

## Global Constraints

- Spec §3.1–§3.6 are the source of truth — when this plan disagrees with the spec, the spec wins.
- `book.js` references 5 `getElementById('fa-X').innerHTML` call sites (`internal/search`/`book.js:227,235,241,287,311`). The `id="fa-X"` attributes must remain; only the *inner* markup changes. Plan handles this in Task 2.
- `internal/fontawesome.TestDeprecationWarningIsOneShot` becomes invalid the moment we delete the package — no replacement test.
- After Task 1 the binary must still build (`go build ./cmd/mdbook`). After Task 2 fixtures regenerate; after Task 3 CSS targets no longer match — purely cosmetic dead-code removal. After Task 4 the repo is clean and the harness diff (`harness/diff.sh`) is reviewable.
- Worker MUST NOT skip the smoke test in Task 1 (build), the regenerate step in Task 2, or the acceptance gate in Task 4.
- `git grep -n 'fontawesome\|font-awesome\|fa-svg\|"fa-'` over `cmd/` `internal/` `theme/` `README.md` `MIGRATION.md` must return zero hits at the end of Task 4.
- All commits land on branch `v1` (current branch). Do not rebase master.

---

## File Structure

| File / directory | Change | Task |
|---|---|---|
| `internal/fontawesome/fontawesome.go` | DELETE | T1 |
| `internal/fontawesome/icons.go` | DELETE | T1 |
| `internal/fontawesome/fontawesome_test.go` | DELETE | T1 |
| `internal/render/render.go` | drop import + `faHelper` + `RegisterFunc("fa", …)` | T1 |
| `internal/tplgotpl/helpers.go` | drop import + `Env.FA` | T1 |
| `internal/html/passes.go` | drop import + `convertFontAwesome` | T1 |
| `internal/html/builder.go` | drop `b.convertFontAwesome()` call | T1 |
| `internal/tplgotpl/prod/index.gohtml` | replace 18 `{{fa …}}` calls + 5 `<template id="fa-X">` bodies | T2 |
| `theme/templates/index.hbs` | same as above | T2 |
| `fixtures/cli/expected/init/theme/index.hbs` | same as above | T2 |
| `theme/css/chrome.css` | drop `.fa-svg`/`.fa-svg:hover` rules | T3 |
| `theme/css/general.css` | drop `.fa-svg svg` + `.blockquote-tag-title .fa-svg` rules | T3 |
| `README.md` | drop `fontawesome/` row + the `fontawesome` mention in line 59 | T4 |
| `MIGRATION.md` | drop §"4. Font Awesome" (line 141–149) | T4 |
| `fixtures/basic/book/*.html` | regenerate | T4 |
| `fixtures/serve/book/*.html` | regenerate | T4 |
| `fixtures/ts-toc-basic_toc/book/**.html` | regenerate | T4 |
| `internal/hbs/testdata/intro.html` | regenerate | T4 |

---

## Task 1: Delete the backend (package + helpers + import sites)

**Files:**
- Delete: `internal/fontawesome/fontawesome.go`
- Delete: `internal/fontawesome/icons.go`
- Delete: `internal/fontawesome/fontawesome_test.go`
- Modify: `internal/render/render.go:21,195,202-222`
- Modify: `internal/tplgotpl/helpers.go:10,74-91`
- Modify: `internal/html/passes.go:6,143-185`
- Modify: `internal/html/builder.go:107`

**Interfaces:** None — this task removes an interface. Tests that consumed the package (`fontawesome_test.go`) are deleted in lockstep.

- [ ] **Step 1: Delete the three files in `internal/fontawesome/`**

```bash
cd "C:\work\mdBook\mdbook-go"
rm internal/fontawesome/fontawesome.go internal/fontawesome/icons.go internal/fontawesome/fontawesome_test.go
rmdir internal/fontawesome 2>/dev/null
```

Expected: no errors; `internal/fontawesome/` directory no longer exists.

- [ ] **Step 2: Strip the call site in `internal/render/render.go`**

In `internal/render/render.go`:
- Delete line 21: `"mdbook-go/internal/fontawesome"` from the import block.
- Delete line 195: `r.RegisterFunc("fa", faHelper)`.
- Delete lines 202–222 (the entire `faHelper` function — `// faHelper … return template.HTML(span), nil\n}` plus its blank-line separator above).

The imports block after this edit must be balanced: the order `fontawesome`/`html`/`search`/`static`/`theme`/`tplgotpl`/`utils` becomes `html`/`search`/`static`/`theme`/`tplgotpl`/`utils` and the imported packages are still used (verify by reading the file).

- [ ] **Step 3: Strip `Env.FA` from `internal/tplgotpl/helpers.go`**

In `internal/tplgotpl/helpers.go`:
- Delete line 10: `"mdbook-go/internal/fontawesome"` from the import block.
- Delete lines 71–91 (the `// FA implements …` doc comment + `func (e *Env) FA(…) (template.HTML, error) { … }` body, ending at the line `return template.HTML(span), nil` followed by `}`).

- [ ] **Step 4: Strip `convertFontAwesome` from `internal/html/passes.go`**

In `internal/html/passes.go`:
- Delete line 6: `"mdbook-go/internal/fontawesome"` from the import block.
- Delete lines 143–185 (the entire `// convertFontAwesome replaces …` doc comment block + `func (b *builder) convertFontAwesome() { … }` body, ending at the `node.Parent.replaceChild(node, span)\n\t}` close brace).

- [ ] **Step 5: Strip the call site in `internal/html/builder.go`**

In `internal/html/builder.go`:
- Delete line 107: `b.convertFontAwesome()`.

- [ ] **Step 6: Build to verify the backend compiles**

Run:
```bash
cd "C:\work\mdBook\mdbook-go"
go build ./...
```
Expected: zero errors. (`go build ./...` will also catch any leftover `fontawesome.` call sites we missed.)

- [ ] **Step 7: Confirm `internal/fontawesome/` is gone**

```bash
cd "C:\work\mdBook\mdbook-go"
ls internal/fontawesome 2>&1
git grep -n 'fontawesome' -- internal/ cmd/
```
Expected: the first command emits `No such file or directory`. The second command returns zero hits **inside `internal/` and `cmd/`** (note: it will still hit helpers/files outside that scope, which is fine for this verification).

- [ ] **Step 8: Commit**

```bash
cd "C:\work\mdBook\mdbook-go"
git add -A
git commit -m "feat(fontawesome): delete package and all production callers

- Drop internal/fontawesome/{fontawesome,icons,fontawesome_test}.go
- Drop faHelper + RegisterFunc(\"fa\", ...) in internal/render/render.go
- Drop Env.FA in internal/tplgotpl/helpers.go
- Drop convertFontAwesome + its call site in internal/html/{passes,builder}.go

Build verified clean via 'go build ./...'. The {{fa ...}} template
helper is now a no-op (FuncMap no longer has it), which is intentional;
template chrome still renders the surrounding <button>/<label>/<a>
elements with their title/aria-label intact.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 2: Strip `{{fa …}}` from the 3 templates, keep `<template id="fa-X">` ids

**Files:**
- Modify: `internal/tplgotpl/prod/index.gohtml` (18 inline `{{fa …}}` calls + 5 `<template id="fa-X">` bodies)
- Modify: `theme/templates/index.hbs` (same surgery)
- Modify: `fixtures/cli/expected/init/theme/index.hbs` (same surgery — this is what `mdbook init` writes into a new user's project)

**Interfaces:** None — pure template rewrites. The 5 `<template id="fa-X">` containers MUST keep their `id` attribute exactly as today because `book.js:227,235,241,287,311` reads `.innerHTML` from them.

- [ ] **Step 1: Edit `internal/tplgotpl/prod/index.gohtml`**

For each of the 18 lines below, replace the `{{fa …}}` token with an empty string (i.e. delete the token and its surrounding whitespace-collapse-safe delimiters — leave a single space if the surrounding tag template would otherwise mash onto adjacent text, otherwise blank):

| Line | Find | Replace with |
|---|---|---|
| 128 | `{{fa "solid" "bars"}}` | ` ` (single space — keeps the `<label>` text readable) |
| 131 | `{{fa "solid" "paintbrush"}}` | ` ` |
| 143 | `{{fa "solid" "magnifying-glass"}}` | ` ` |
| 151 | `{{fa "solid" "print" "print-button"}}` | ` ` |
| 156 | `{{fa .GitRepositoryIconClass .GitRepositoryIcon}}` | ` ` |
| 161 | `{{fa "solid" "pencil" "git-edit-button"}}` | ` ` |
| 172 | `{{fa "solid" "spinner" "fa-spin"}}` | ` ` |
| 198, 221 | `{{fa "solid" "angle-right"}}` | `›` (single Unicode arrow character) |
| 200, 207, 223, 230 | `{{fa "solid" "angle-left"}}` | `‹` |
| 209, 232 | `{{fa "solid" "angle-left"}}` | `‹` |

For the 5 `<template id="fa-X">…</template>` blocks (lines 238–242), keep the line structure but zero the inner:

| Line | Find | Replace with |
|---|---|---|
| 238 | `<template id=fa-eye>{{fa "solid" "eye"}}</template>` | `<template id="fa-eye"></template>` |
| 239 | `<template id=fa-eye-slash>{{fa "solid" "eye-slash"}}</template>` | `<template id=fa-eye-slash"></template>` ← confirm bracket; the find string above closes one quote; replace exactly the line as-is, ending with `</template>` and starting with the template open tag |
| 240 | `<template id=fa-copy>{{fa "regular" "copy"}}</template>` | `<template id=fa-copy"></template>` |
| 241 | `<template id=fa-play>{{fa "solid" "play"}}</template>` | `<template id=fa-play"></template>` |
| 242 | `<template id=fa-clock-rotate-left>{{fa "solid" "clock-rotate-left"}}</template>` | `<template id=fa-clock-rotate-left"></template>` |

(The above replace-with lines are illustrative — read the source first and construct the exact new lines, then commit per-tool edits. Use the Edit tool with `old_string` = full current line and `new_string` = full new line.)

Verify after the edit:
```bash
cd "C:\work\mdBook\mdbook-go"
grep -n '{{fa' internal/tplgotpl/prod/index.gohtml
grep -n 'id=fa-' internal/tplgotpl/prod/index.gohtml
```
Expected: first command emits no matches; second command emits 5 lines (one per template id).

- [ ] **Step 2: Mirror the change in `theme/templates/index.hbs`**

Apply the same line-by-line replacement to lines 268–272 (the `<template id="fa-X">` blocks) and the equivalent `{{fa …}}` call sites between lines 158–242 of `theme/templates/index.hbs`. Run:
```bash
cd "C:\work\mdBook\mdbook-go"
grep -nc '{{fa' theme/templates/index.hbs
```
Expected: 0.

- [ ] **Step 3: Mirror the change in `fixtures/cli/expected/init/theme/index.hbs`**

Apply the same surgery; this file is the scaffold that `mdbook init` writes into a fresh book project, so it must stay consistent with what `prod/index.gohtml` does. Run:
```bash
cd "C:\work\mdBook\mdbook-go"
grep -nc '{{fa' fixtures/cli/expected/init/theme/index.hbs
```
Expected: 0.

- [ ] **Step 4: Build binary, regen `fixtures/basic/book/`, smoke-check**

```bash
cd "C:\work\mdBook\mdbook-go"
go build -o ./bin/mdbook-go.exe ./cmd/mdbook
./bin/mdbook-go.exe build -dir ./fixtures/basic
```

Then:
```bash
grep -c 'class=fa-svg' fixtures/basic/book/intro.html
grep -c '<template id=fa-' fixtures/basic/book/intro.html
grep -c 'class="markdown-body"' fixtures/basic/book/intro.html
```

Expected output:
- `0` (no more `<span class=fa-svg>` in chrome)
- `>=1` (the 5 `<template id="fa-X">` containers survived)
- `1` (the `<main class="markdown-body">` marker from Task 1's smoke test)

- [ ] **Step 5: Spot-check the angular-nav Unicode arrows**

```bash
cd "C:\work\mdBook\mdbook-go"
grep -n '›\|‹' fixtures/basic/book/intro.html
```
Expected: at least 2 lines (the wide-nav + mobile-nav arrow slots). If 0 hits, Task 2's "replace with Unicode" sub-step wasn't applied — investigate.

- [ ] **Step 6: Commit**

```bash
cd "C:\work\mdBook\mdbook-go"
git add internal/tplgotpl/prod/index.gohtml theme/templates/index.hbs fixtures/cli/expected/init/theme/index.hbs
git add fixtures/basic/book/  # the regenerated intro.html etc. (only commit these — defer serve/ts-toc/hbs goldens to Task 4)
git commit -m "feat(templates): drop {{fa ...}} chrome, empty fa-X templates

The hbs engine (theme/templates/index.hbs) and the mdbook init
scaffold (fixtures/cli/expected/init/theme/index.hbs) get the same
surgery as internal/tplgotpl/prod/index.gohtml.

- 18 inline {{fa ...}} calls → empty (or Unicode '‹' / '›' for prev/next)
- 5 <template id=\"fa-X\"> bodies → empty (ids retained for book.js)

book.js:227,235,241,287,311 keep working: their document.getElementById()
reads return ''. The block-collapse, code-copy, playground-run, and
history-reset buttons become textless but functional.

fixtures/basic/book/intro.html regenerated and shipped with this commit;
remaining fixtures (serve, ts-toc, hbs testdata) regenerate in Task 4.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 3: Strip dead `.fa-svg*` CSS selectors

**Files:**
- Modify: `theme/css/chrome.css`
- Modify: `theme/css/general.css`

**Interfaces:** None — cosmetic dead-code removal. Existing selectors are pure `font-awesome`-only: they target `<span class="fa-svg"><svg>…</svg></span>` elements that no longer exist.

- [ ] **Step 1: Edit `theme/css/chrome.css`**

Five selectors to clean. After each edit, verify the file still parses (no dangling commas in `selector-list`).

Specific changes:

- Line 59–66: rule `@media` parent block contains `#mdbook-menu-bar .fa-svg, #mdbook-menu-bar .icon-button`. Keep the rule, drop the `.fa-svg` selector:
  ```diff
  - #mdbook-menu-bar .fa-svg, #mdbook-menu-bar .icon-button {
  + #mdbook-menu-bar .icon-button {
  ```
- Lines 68–70: under the `max-width: 420px` media query, same selector. Same edit:
  ```diff
  - #mdbook-menu-bar .fa-svg, #mdbook-menu-bar .icon-button {
  + #mdbook-menu-bar .icon-button {
  ```
- Line 79: `.icon-button .fa-svg { … }` (the inner-SVG sizing block). Delete the entire rule — `.icon-button` does not need an inner-style block once the icon span is gone. Find the opening `{` and the matching `}` (the rule closes on line ~85) and remove.
- Line 121: in the color rules block — `.menu-bar a .fa-svg { color: var(--icons); }`. Delete just the `.menu-bar a .fa-svg { … }` block.
- Lines 125–128: hover rules split — `.menu-bar .fa-svg:hover, .menu-bar .icon-button:hover, .nav-chapters:hover, .mobile-nav-chapters .fa-svg:hover { color: var(--icons-hover); }`. Drop the `.menu-bar .fa-svg:hover` and `.mobile-nav-chapters .fa-svg:hover` selectors but keep the rule for `.menu-bar .icon-button:hover, .nav-chapters:hover`:
  ```diff
  - .menu-bar .fa-svg:hover,
  - .menu-bar .icon-button:hover,
  - .nav-chapters:hover,
  - .mobile-nav-chapters .fa-svg:hover {
  + .menu-bar .icon-button:hover,
  + .nav-chapters:hover {
  ```

After edits, verify no `.fa-svg` selectors remain:
```bash
cd "C:\work\mdBook\mdbook-go"
grep -n '\.fa-svg' theme/css/chrome.css
```
Expected: no matches.

- [ ] **Step 2: Edit `theme/css/general.css`**

Two rules:

- Lines 288–293: `.fa-svg svg { width: 1em; height: 1em; fill: currentColor; margin-bottom: -0.1em; }`. Delete the entire block.
- Lines 407–411: `.blockquote-tag-title .fa-svg { fill: currentColor; margin-right: 8px; }`. Delete the entire block.

Verify:
```bash
cd "C:\work\mdBook\mdbook-go"
grep -n '\.fa-svg' theme/css/general.css
```
Expected: no matches.

- [ ] **Step 3: Build + regen + smoke check**

```bash
cd "C:\work\mdBook\mdbook-go"
go build -o ./bin/mdbook-go.exe ./cmd/mdbook
./bin/mdbook-go.exe build -dir ./fixtures/basic
grep -c 'fa-svg' fixtures/basic/book/css/chrome-*.css fixtures/basic/book/css/general-*.css
```
Expected:
- Build exits 0.
- The grep counts 0 — both hosted CSS files now contain zero `.fa-svg` references.

- [ ] **Step 4: Verify chrome visually**

Open `fixtures/basic/book/index.html` in a browser (or `xdg-open`/`start`); confirm:
- The menu bar (top) still has Sidebar / Theme / Search / Print buttons in their expected positions — even if they no longer carry icon glyphs.
- The "Toggle Table of Contents" label still appears on hover.

- [ ] **Step 5: Commit**

```bash
cd "C:\work\mdBook\mdbook-go"
git add theme/css/chrome.css theme/css/general.css
git commit -m "feat(css): drop dead .fa-svg* selectors

chrome.css + general.css had ~10 rules matching .fa-svg / .fa-svg:hover /
.fa-svg svg. With fontawesome gone, none of these selectors ever match
again. Delete the rules; keep the surrounding blocks intact.

- chrome.css: 5 affected rules tightened to .icon-button / .menu-bar
  hover selectors
- general.css: 2 standalone rules deleted

Verified via 'grep -c fa-svg fixtures/basic/book/css/*.css' → 0.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 4: Regenerate remaining fixtures, update docs, final acceptance gate

**Files:**
- Modify (regenerate): `fixtures/serve/book/*.html`
- Modify (regenerate): `fixtures/ts-toc-basic_toc/book/**/*.html`
- Modify (regenerate): `internal/hbs/testdata/intro.html`
- Modify: `README.md:36,59`
- Modify: `MIGRATION.md:141-149`

**Interfaces:** None. This task runs end-to-end build commands and trims documentation paragraphs.

- [ ] **Step 1: Regenerate `fixtures/serve/book/`**

```bash
cd "C:\work\mdBook\mdbook-go"
ls fixtures/serve/
```
Look for `src/` and `book.toml`. Then:
```bash
./bin/mdbook-go.exe build -dir ./fixtures/serve
```
Expected: exits 0; `fixtures/serve/book/*.html` is overwritten with the new shape (no `fa-svg`, populated `<main class="markdown-body">`).

- [ ] **Step 2: Regenerate `fixtures/ts-toc-basic_toc/book/`**

```bash
cd "C:\work\mdBook\mdbook-go"
./bin/mdbook-go.exe build -dir ./fixtures/ts-toc-basic_toc
```
Expected: exits 0. The fixture has nested chapters, so expect ~10 regenerated files.

- [ ] **Step 3: Regenerate the hbs golden**

```bash
cd "C:\work\mdBook\mdbook-go"
go test ./internal/hbs/ -run HBS -update
```
If the test suite uses a different flag, check `internal/hbs/hbs_golden_test.go` for the update flag name. Expected: the `internal/hbs/testdata/intro.html` golden is rewritten to the new shape.

- [ ] **Step 4: Update `README.md`**

In `README.md`:
- Line 36: delete the entire line `│   ├── fontawesome/   Font Awesome icon SVGs`.
- Line 59: delete `internal/fontawesome` from the list of golden-test packages — find the substring `, \`internal/fontawesome\`` and remove it, leaving the list as `\`internal/hbs\`, \`internal/search\`, \`internal/html\``.

Verify:
```bash
cd "C:\work\mdBook\mdbook-go"
grep -n 'fontawesome' README.md
```
Expected: no matches.

- [ ] **Step 5: Delete §"4. Font Awesome" from `MIGRATION.md`**

In `MIGRATION.md`, lines 141–149 contain the deprecated-feature subsection ("4. Font Awesome — *deprecated*. The Go port embeds only 15…"). Read the surrounding `1.`/`2.`/`3.` headings first to confirm the section's numbering; if the file already lost a previous subsection, use the right line range. Delete lines 141 to the next blank line (or the next `N. ` heading, whichever comes first). Result: the migration guide no longer has a "Font Awesome" subsection.

Verify:
```bash
cd "C:\work\mdBook\mdbook-go"
grep -n 'Font Awesome\|fontawesome\|font-awesome' MIGRATION.md
```
Expected: no matches.

- [ ] **Step 6: Run the full test suite**

```bash
cd "C:\work\mdBook\mdbook-go"
go test ./...
```
Expected: passes. The only suites that can be skipped-but-not-broken:
- `internal/hbs/hbs_golden_test.go` — must pass (we updated the golden in Step 3).
- `internal/html/markdown_golden_test.go` — must pass; if a golden mentions `<i class="fa-…">` content, update the test data in this step.
- Any rust-testsuite integration test (excluded from this workstream per spec).

- [ ] **Step 7: Final acceptance gate**

```bash
cd "C:\work\mdBook\mdbook-go"
git grep -n 'fontawesome\|font-awesome\|fa-svg\|"fa-' cmd/ internal/ theme/ README.md MIGRATION.md
git grep -n 'fontawesome' internal/fontawesome 2>&1 || true  # expected: 'No such file or directory'
ls -la bin/mdbook-go.exe
./bin/mdbook-go.exe build -dir ./fixtures/basic
grep -c '<span class=fa-svg' fixtures/basic/book/intro.html
grep -c 'class="markdown-body"' fixtures/basic/book/intro.html
grep -c '<main' fixtures/basic/book/intro.html
```

Expected output:
- `git grep` over the 4 files above: 0 hits.
- `internal/fontawesome`: directory does not exist.
- `bin/mdbook-go.exe` exists (binary size should be ~700KB smaller than before Task 1; spot-compare with `git show HEAD~4:bin/mdbook-go.exe | wc -c` if you want a number).
- `grep -c '<span class=fa-svg' fixtures/basic/book/intro.html` → `0`.
- `grep -c 'class="markdown-body"' fixtures/basic/book/intro.html` → `1`.
- `grep -c '<main' fixtures/basic/book/intro.html` → `1`.

If any of these fails, investigate before committing.

- [ ] **Step 8: Commit the regenerated fixtures + docs**

```bash
cd "C:\work\mdBook\mdbook-go"
git add fixtures/serve/book fixtures/ts-toc-basic_toc/book internal/hbs/testdata/intro.html README.md MIGRATION.md
git commit -m "feat(fixtures,docs): regenerate goldens after fontawesome removal

- fixtures/serve/book/* regenerated
- fixtures/ts-toc-basic_toc/book/**/*.html regenerated
- internal/hbs/testdata/intro.html regenerated
- README.md: drop fontawesome directory tree row and the line that
  lists it as a regression package
- MIGRATION.md: drop the deprecated Font Awesome helper subsection

Final acceptance: 'git grep fontawesome fa-svg font-awesome \"fa-''
over cmd/internal/theme/README.md/MIGRATION.md returns 0 hits.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Self-Review

1. **Spec coverage:**
   - §3.1 (delete package) → Task 1
   - §3.2 (production callers) → Task 1
   - §3.3 (templates) → Task 2
   - §3.4 (CSS) → Task 3
   - §3.5 (docs) → Task 4
   - §3.6 (fixture regen) → Task 2 (basic, smoke gate) + Task 4 (serve/ts-toc/hbs golden + final acceptance)
   - §4 (data flow) → this plan doesn't enforce new data flow, only enforces that the old one is gone. Covered by the spec; no implementation task needed beyond Tasks 1–4.
   - §5 (failure modes) → Task 2's `book.js` empty innerHTML behaviour is exercised in the smoke step; the `<i class="fa-…">` markdown case is left as "acceptable per spec" with no mitigation by design.
   - §6 (testing) → Steps 1/2/3/4 in Task 4 plus the per-task build smoke checks.
   - §7 (acceptance) → Task 4 Step 7.
   - §8 (out of scope) → not implemented (correctly).

2. **Placeholder scan:** No "TBD"/"TODO"/"implement later" — every step has exact commands or exact code edits.

3. **Type consistency check:** The plan doesn't introduce new public types; it deletes them. `faHelper`/`Env.FA`/`convertFontAwesome` references are removed in lockstep in Task 1. No leftover identifiers.

4. **Risk check:** Task 2's "replace with Unicode '‹'/'›'" row table has two `›` rows and a few `‹` rows — that's intentional (multiple call sites use the same icon). The replacement values are repeatable; a typo in one row won't cascade.

5. **Out-of-scope guard:** No mention of `internal/hbs` rendering path promotion, no theme-asset pipeline building, no Rust testsuite work. All correctly left to future work.
