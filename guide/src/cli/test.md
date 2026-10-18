# The test command

When writing a book, you sometimes need to automate some tests. For example,
[The Rust Programming Book](https://doc.rust-lang.org/stable/book/) uses a lot
of code examples that could get outdated. Therefore it is very important for
them to be able to automatically test these code examples.

mdBook supports a `test` command that will run all available tests in a book. At
the moment, only Rust tests are supported.

#### Disable tests on a code block

rustdoc doesn't test code blocks which contain the `ignore` attribute:

````text
    ```rust,ignore
    fn main() {}
    ```
````

rustdoc also doesn't test code blocks which specify a language other than Rust:

````text
    ```markdown
    **Foo**: _bar_
    ```
````

rustdoc *does* test code blocks which have no language specified:

````text
    ```
    This is going to cause an error!
    ```
````

#### Declaring dependencies

In the book's root directory, you may place a `Cargo.toml` manifest declaring
properties such as Rust edition and external dependencies. Remember to add an empty
`src/lib.rs` file to keep the structure same as in a Rust library. You may add your own
helper code to the `src/lib.rs` file and refer to it in your snippets.

#### Specify a directory

The `test` command can take a directory as an argument to use as the book's root
instead of the current working directory.

```bash
mdbook test path/to/book
```

#### `--library-path` (DEPRECATED)

The `--library-path` (`-L`) option allowed you to add directories to the library
search path used by `rustdoc` when it builds and tests the examples. It is now
deprecated.

#### `--chapter`

The `--chapter` (`-c`) option allows you to test a specific chapter of the
book using the chapter name, or matching against it or against the path
to the chapter.
