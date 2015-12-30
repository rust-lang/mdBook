# The watch command

The `watch` command is useful when you want your book to be rendered on every file change.
You could repeatedly issue `mdbook build` every time a file is changed. But using `mdbook watch` once will watch your files and will trigger a build automatically whenever you modify a file.

#### Specify a directory

Like `init` and `build`, `watch` can take a directory as argument to use instead of the
current working directory.

```bash
mdbook watch path/to/book
```


-----

***note:*** *the `watch` command has not gotten a lot of testing yet, there could be some rough edges. If you discover a problem, please report it [on Github](https://github.com/azerupi/mdBook/issues)*
