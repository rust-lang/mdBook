# mdBook

Personal implementation of Gitbook in Rust

**This is still a work in progress...**

### cli tool

#### init

If you run `mdbook init` in a directory, it will create a couple of folders and files you can start with.
This is the strucutre it creates at the moment:
```
book-test/
├── book
└── src
    ├── chapter_1.md
    └── SUMMARY.md
```
`book` and `src` are both directories. `src` contains the markdown files that will be used to render the ouput to the `book` directory.

You can also pass a directory to `mdbook init` to use instead of the current directory:
```
mdbook init path/to/my/book
```

<sup>**Not implemented yet:** using `--theme` flag will create a theme folder with the default theme in `src` so that you can customize it.</sup>

#### build

Use `mdbook build` in the directory to render the book. You can also give a path as argument to use instead of the current directory.


### lib

Aside the command-line tool, this crate can also be used as a library. 

-------------------------------------------------------

For more information about what is left on my to-do list, check the issue tracker
