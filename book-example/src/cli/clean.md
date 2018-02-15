# The clean command

The clean command is used to delete the generated book and any other build
artifacts.

```bash
mdbook clean
```

It will try to delete the built book. If a path is provided, it will be used.

#### Specify a directory

Like `init`, the `clean` command can take a directory as an argument to use
instead of the normal build directory.

```bash
mdbook clean --dest-dir=path/to/book
```

`path/to/book` could be absolute or relative.