# Creating a Book

Once you have the `mdbook` CLI tool installed, you can use it to create and render a book.

## Initializing a book

The `mdbook init` command will create a new directory containing an empty book for you to get started.
Give it the name of the directory that you want to create:

```sh
mdbook init my-first-book
```

It will ask a few questions before generating the book.
After answering the questions, you can change the current directory into the new book:

```sh
cd my-first-book
```

There are several ways to render a book, but one of the easiest methods is to use the `serve` command, which will build your book and start a local webserver:

```sh
mdbook serve --open
```

The `--open` option will open your default web browser to view your new book.
You can leave the server running even while you edit the content of the book, and `mdbook` will automatically rebuild the output *and* automatically refresh your web browser.

Check out the [CLI Guide](../cli/index.html) for more information about other `mdbook` commands and CLI options.

## Anatomy of a book

A book is built from several files which define the settings and layout of the book.

### `book.toml`

In the root of your book, there is a `book.toml` file which contains settings for describing how to build your book.
This is written in the [TOML markup language](https://toml.io/).
The default settings are usually good enough to get you started.
When you are interested in exploring more features and options that mdBook provides, check out the [Configuration chapter](../format/configuration/index.html) for more details.

A very basic `book.toml` can be as simple as this:

```toml
[book]
title = "My First Book"
```

### `SUMMARY.md`

The next major part of a book is the summary file located at `src/SUMMARY.md`.
This file contains a list of all the chapters in the book.
Before a chapter can be viewed, it must be added to this list.

Here's a basic summary file with a few chapters:

```md
# Summary

[Introduction](README.md)

- [My First Chapter](my-first-chapter.md)
- [Nested example](nested/README.md)
    - [Sub-chapter](nested/sub-chapter.md)
```

Try opening up `src/SUMMARY.md` in your editor and adding a few chapters.
If any of the chapter files do not exist, `mdbook` will automatically create them for you.

For more details on other formatting options for the summary file, check out the [Summary chapter](../format/summary.md).

### Source files

The content of your book is all contained in the `src` directory.
Each chapter is a separate Markdown file.
Typically, each chapter starts with a level 1 heading with the title of the chapter.

```md
# My First Chapter

Fill out your content here.
```

The precise layout of the files is up to you.
The organization of the files will correspond to the HTML files generated, so keep in mind that the file layout is part of the URL of each chapter.

While the `mdbook serve` command is running, you can open any of the chapter files and start editing them.
Each time you save the file, `mdbook` will rebuild the book and refresh your web browser.

Check out the [Markdown chapter](../format/markdown.md) for more information on formatting the content of your chapters.

All other files in the `src` directory will be included in the output.
So if you have images or other static files, just include them somewhere in the `src` directory.

## Publishing a book

Once you've written your book, you may want to host it somewhere for others to view.
The first step is to build the output of the book.
This can be done with the `mdbook build` command in the same directory where the `book.toml` file is located:

```sh
mdbook build
```

This will generate a directory named `book` which contains the HTML content of your book.
You can then place this directory on any web server to host it.

For more information about publishing and deploying, check out the [Continuous Integration chapter](../continuous-integration.md) for more.