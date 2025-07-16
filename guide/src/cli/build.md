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
book. Relative paths are interpreted relative to the book's root directory. If
not specified it will default to the value of the `build.build-dir` key in
`book.toml`, or to `./book`.

#### `--backend`

By default, all backends configured in the `book.toml` config file will be executed.
If this flag is given, only the specified backend will be run. This flag 
may be given multiple times to run multiple backends. Providing a name of
a backend that is not configured results in an error. For more information
about backends, see [here](./format/configuration/renderers.md).

-------------------

***Note:*** *The build command copies all files (excluding files with `.md` extension) from the source directory
into the build directory.*
