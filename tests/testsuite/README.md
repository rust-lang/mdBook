# Testsuite

## Introduction

This is the main testsuite for exercising all functionality of mdBook.

Tests should be organized into modules based around major features. Tests should use `BookTest` to drive the test. `BookTest` will set up a temp directory, and provides a variety of methods to help create a build books.

## Basic structure of a test

Using `BookTest`, you typically use it to copy a directory into a temp directory, and then run mdbook commands in that temp directory. You can run the `mdbook` executable, or use the mdbook API to perform whatever tasks you need. Running the executable has the benefit of being able to validate the console output.

See `build::basic_build` for a simple test example. I recommend reviewing the methods on `BookTest` to learn more, and reviewing some of the existing tests to get a feel for how they are structured.

For example, let's say you are creating a new theme test. In the `testsuite/theme` directory, create a new directory with the book source that you want to exercise. At a minimum, this needs a `src/SUMMARY.md`, but often you'll also want `book.toml`. Then, in `testsuite/theme.rs`, add a test with `BookTest::from_dir("theme/mytest")`, and then use the methods to perform whatever actions you want.

`BookTest` is designed to be able to chain a series of actions. For example, you can do something like:

```rust
BookTest::from_dir("theme/mytest")
    .build()
    .check_main_file("book/index.html", str![["file contents"]])
    .change_file("src/index.md", "new contents")
    .build()
    .check_main_file("book/index.html", str![["new contents"]]);
```

## Snapbox

The testsuite uses [`snapbox`] to drive most of the tests. This library provides the ability to compare strings using a variety of methods. These strings are written in the source code using either the [`str!`] or [`file!`] macros.

The magic is that you can set the `SNAPSHOTS=overwrite` environment variable, and snapbox will automatically update the strings contents of `str!`, or the file contents of `file!`. This makes it easier to update tests. Snapbox provides nice diffing output, and quite a few other features.

Expected contents can have wildcards like `...` (matches any lines) or `[..]` (matches any characters on a line). See [snapbox filters] for more info and other filters.

Typically when writing a test, I'll just start with an empty `str!` or `file!`, and let snapbox fill it in. Then I review the contents to make sure they are what I expect.

Note that there is some normalization applied to the strings. See `book_test::assert` for how some of these normalizations happen.

[`snapbox`]: https://docs.rs/snapbox/latest/snapbox/
[`str!`]: https://docs.rs/snapbox/latest/snapbox/macro.str.html
[`file!`]: https://docs.rs/snapbox/latest/snapbox/macro.file.html
[snapbox filters]: https://docs.rs/snapbox/latest/snapbox/assert/struct.Assert.html#method.eq
