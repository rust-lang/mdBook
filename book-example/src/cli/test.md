# The test command

When writing a book, you sometimes need to automate some tests. For example, [The Rust Programming Book](https://doc.rust-lang.org/stable/book/) uses a lot of code examples that could get outdated.
Therefore it is very important for them to be able to automatically test these code examples.

mdBook supports a `test` command that will run all available tests in mdBook. At the moment, only one test is available:
*"Test Rust code examples using Rustdoc"*, but I hope this will be expanded in the future to include more tests like:

- checking for broken links
- checking for unused files
- ...

In the future I would like the user to be able to enable / disable test from the `book.json` configuration file and support custom tests.

**How to use it:**
```bash
$ mdbook test
[*]: Testing file: "/mdBook/book-example/src/README.md”
```
