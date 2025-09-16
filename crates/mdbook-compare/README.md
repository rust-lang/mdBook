# mdbook-compare

This is a simple utility to compare the output of two different versions of mdbook.

To use this:

1. Install [`tidy`](https://www.html-tidy.org/).
2. Install or build the initial version of mdbook that you want to compare.
3. Install or build the new version of mdbook that you want to compare.
4. Run `mdbook-compare` with the arguments to the mdbook executables and the books to build.

```sh
cargo run --manifest-path /path/to/mdBook/Cargo.toml -p mdbook-compare -- \
    /path/to/orig/mdbook /path/to/my-book /path/to/new/mdbook /path/to/my-book
```

It takes two separate paths for the book to use for "before" and "after" in case you need to customize the book to run on older versions. If you don't need that, then you can use the same directory for both the before and after.

`mdbook-compare` will do the following:

1. Clean up any book directories.
2. Build the book with the first mdbook.
3. Build the book with the second mdbook.
4. The output of those two commands are stored in directories called `compare1` and `compare2`.
5. The HTML in those directories is normalized using `tidy`.
6. Runs `git diff` to compare the output.
