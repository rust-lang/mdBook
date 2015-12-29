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

Like `init`, the `build` command can take a directory as argument to use instead of the
current working directory.

```bash
mdbook build path/to/book
```

-------------------

***note:*** *make sure to run the build command in the root directory and not in the source directory*
