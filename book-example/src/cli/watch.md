# The watch command

The watch command is useful when you want your book to be rendered on every file change.
You could issue `mdbook build` everytime you change a file. But using `mdbook watch` once will watch your files and will trigger a build whenever you modify a file.

#### Specify a directory

Like `init` and `build`, `watch` can take a directory as argument to use instead of the
current working directory.

```
mdbook watch path/to/book
```
