# The init command

The init command, used like this:

```
mdbook init
```

Will create a couple of files and directories in the working directory so that you can
spend more time writing your book and less setting it up.

The files set up for you are the following:
```
book-test/
├── book
└── src
    ├── chapter_1.md
    └── SUMMARY.md
```

The `src` directory is were you write your book in markdown. It contains all the source files,
configuration files, etc.

The `book` directory is where your book is rendered. All the output is ready to be uploaded
to a serer to be seen by the internet.

The `SUMMARY.md` file is the most important file, it's the skeleton of your book.
It's so important that it has it's own [chapter](../format/summary.html).

#### Specify a directory

When using the init command, you can also specify a directory, instead of using the current directory,
by appending a path to the command:

```
mdbook init path/to/book
```

## --theme

When you use the `--theme` argument, the default theme will be copied into a directory
called `theme` in your source directory so that you can modify it.

The theme is selectively overwritten, this means that if you don't want to overwrite a
specific file, just delete it and the default file will be used.

## Not yet implemented

In the future I would like `mdBook init` to be able to:

- Generate files that are in `SUMMARY.md`. If the user has already created a `SUMMARY.md` file and added some entries but did
not create the corresponding files, init command should create the files for him.
