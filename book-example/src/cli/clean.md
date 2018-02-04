# The clean command

The `clean` command is used to delete directory where built book is located.

```bash
mdbook clean
```

It will try to delete built book. If you pass path to directory it will be used
instead.


#### Specify a directory

Like `init`, the `clean` command can take a directory as an argument to use
instead of the standard directory where generated book is placed once built.

```bash
mdbook clean --dest-dir=path/to/book
```

`path/to/book` could be absolute or relative.