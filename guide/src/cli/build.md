# The build command

The build command is used to render your book:

```bash
mdbook build
```

It will try to parse your `SUMMARY.md` file to understand the structure of your
book and fetch the corresponding files. Note that this will also create files
mentioned in `SUMMARY.md` which are not yet present.

The rendered output will maintain the same directory structure as the source for
convenience. Large books will therefore remain structured when rendered.

#### Specify a directory

The `build` command can take a directory as an argument to use as the book's
root instead of the current working directory.

```bash
mdbook build path/to/book
```

#### `--open`

When you use the `--open` (`-o`) flag, mdbook will open the rendered book in
your default web browser after building it.

#### `--dest-dir`

The `--dest-dir` (`-d`) option allows you to change the output directory for the
book. Relative paths are interpreted relative to the current directory. If
not specified it will default to the value of the `build.build-dir` key in
`book.toml`, or to `./book`.

#### `--copy-exclude-extensions`

The `--copy-exclude-extensions` option allows you to exclude specific file extensions
when copying files from the source directory to the build directory. This is useful when
your source directory contains symlinks to other directories with files you don't want to
include in the output.

Provide a comma-separated list of extensions (without dots):

```bash
mdbook build --copy-exclude-extensions rs,toml,lock
```

This supplements any extensions configured in `book.toml` via the
`output.html.copy-exclude-extensions` setting.

-------------------

***Note:*** *The build command copies all files from the source directory into the build directory.
You can exclude specific file extensions using the `--copy-exclude-extensions` flag or the
`output.html.copy-exclude-extensions` configuration option.*
