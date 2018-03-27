# The watch command

The `watch` command is useful when you want your book to be rendered on every file change.
You could repeatedly issue `mdbook build` every time a file is changed. But using `mdbook watch` once will watch your files and will trigger a build automatically whenever you modify a file.

#### Specify a directory

Like `init` and `build`, `watch` can take a directory as an argument to use
instead of the current working directory.

```bash
mdbook watch path/to/book
```

#### --open

When you use the `--open` (`-o`) option, mdbook will open the rendered book in
your default web browser.

#### --dest-dir

The `--dest-dir` (`-d`) option allows you to change the output directory for your book.

-----

***note:*** *the `watch` command has not gotten a lot of testing yet, there could be some rough edges. If you discover a problem, please report it [on Github](https://github.com/rust-lang-nursery/mdBook/issues)*
