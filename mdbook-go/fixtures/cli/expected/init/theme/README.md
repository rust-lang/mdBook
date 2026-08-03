# Theme

This directory is created by `mdbook init -theme`. With the embedded
default theme it also contains `book.js`, `index.hbs`, `highlight.css`,
`highlight.js`, `favicon.png`, `favicon.svg`, plus the `css/` and
`fonts/` subdirectories. The exact file list mirrors what
`internal/theme.Copy` writes when `printEnable=true`.

The files are intentionally not checked in here; the `expected/init`
tree is only the *skeleton* the user is expected to see (the README
itself is no longer written — `init -theme` writes real theme assets
now). Run `diff -r` against a freshly-initialised directory to verify.
