# The watch command

The `watch` command is useful when you want your book to be rendered on every
file change. You could repeatedly issue `mdbook build` every time a file is
changed. But using `mdbook watch` once will watch your files and will trigger a
build automatically whenever you modify a file; this includes re-creating
deleted files still mentioned in `SUMMARY.md`!

#### Specify a directory

The `watch` command can take a directory as an argument to use as the book's
root instead of the current working directory.

```bash
mdbook watch path/to/book
```

#### `--open`

When you use the `--open` (`-o`) option, mdbook will open the rendered book in
your default web browser.

#### `--dest-dir`

The `--dest-dir` (`-d`) option allows you to change the output directory for the
book. Relative paths are interpreted relative to the book's root directory. If
not specified it will default to the value of the `build.build-dir` key in
`book.toml`, or to `./book`.

{{#include arg-watcher.md}}

#### Specify exclude patterns

The `watch` command will not automatically trigger a build for files listed in
the `.gitignore` file in the book root directory. The `.gitignore` file may
contain file patterns described in the [gitignore
documentation](https://git-scm.com/docs/gitignore). This can be useful for
ignoring temporary files created by some editors.

_Note: Only `.gitignore` from book root directory is used. Global
`$HOME/.gitignore` or `.gitignore` files in parent directories are not used._

#### `--backend`

By default, all backends configured in the `book.toml` config file will be executed.
If this flag is given, only the specified backend will be run. This flag 
may be given multiple times to run multiple backends. Providing a name of
a backend that is not configured results in an error. For more information
about backends, see [here](./format/configuration/renderers.md).
