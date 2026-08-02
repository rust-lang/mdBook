# Known M1 differences (Rust vs Go)

The diff harness exits non-zero on the `basic` fixture at M1. Every line
below is **expected** at M1 and will be removed as later milestones land.

## Missing files in the Go output

- `404.html` — M2.
- `print.html` — M2.
- `toc.html` — M2.
- `searchindex.js` — M2.
- `FontAwesome` resources, CSS, JS — M2.

## Rust-only markup in shared files

The Rust output for each chapter embeds a Handlebars-rendered theme:

- `<head>` contains `<link rel="stylesheet">` to a hashed CSS file.
- `<body>` contains the menu bar, sidebar, chapter navigation, and the
  JavaScript bootstrap.
- The page footer contains `window.playground_copyable = true;` and other
  theme initialisation snippets.

The Go output at M1 is intentionally a minimal envelope:

```html
<!doctype html>
<html><head><meta charset="utf-8"><title>NAME - TITLE</title></head>
<body><main>BODY</main></body></html>
```

## Body content

The visible text of each chapter matches between Rust and Go:

- chapter titles
- paragraph text
- code blocks
- tables
- list items

The Rust output also emits "Basic Fixture" in the header (book title) and
side navigation. The Go output includes the book title only in `<title>`.

## Closure plan

| M  | File / feature expected to land            |
|----|--------------------------------------------|
| M2 | 404.html, print.html, toc.html             |
| M2 | theme resources, sidebar markup, footer JS |
| M2 | searchindex.js, redirects                  |
| M3 | preprocessor / renderer JSON protocol      |
| M5 | live reload, watch                         |
