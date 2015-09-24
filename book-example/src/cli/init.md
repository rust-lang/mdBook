# The init command

The `init` command is used like this:

```
mdbook init
```

It will create a couple of files and directories in the working directory so that you can
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
to a server to be seen by the internet.

The `SUMMARY.md` file is the most important file, it's the skeleton of your book and is discussed in more detail in another  [chapter](../format/summary.html).

When a `SUMMARY.md` file already exists, the `init` command will generate the files according to the paths used in the `SUMMARY.md`

#### Specify a directory

When using the `init` command, you can also specify a directory, instead of using the current working directory,
by appending a path to the command:

```
mdbook init path/to/book
```

## --theme

When you use the `--theme` argument, the default theme will be copied into a directory
called `theme` in your source directory so that you can modify it.

The theme is selectively overwritten, this means that if you don't want to overwrite a
specific file, just delete it and the default file will be used.
