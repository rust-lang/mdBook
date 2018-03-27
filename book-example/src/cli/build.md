# The build command

The build command is used to render your book:

```bash
mdbook build
```

It will try to parse your `SUMMARY.md` file to understand the structure of your book
and fetch the corresponding files.

The rendered output will maintain the same directory structure as the source for
convenience. Large books will therefore remain structured when rendered.

#### Specify a directory

Like `init`, the `build` command can take a directory as an argument to use
instead of the current working directory.

```bash
mdbook build path/to/book
```

#### --open

When you use the `--open` (`-o`) option, mdbook will open the rendered book in
your default web browser after building it.

#### --dest-dir

The `--dest-dir` (`-d`) option allows you to change the output directory for your book.

-------------------

***note:*** *make sure to run the build command in the root directory and not in the source directory*
