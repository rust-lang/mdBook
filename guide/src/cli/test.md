# The test command

When writing a book, you may want to provide some code samples,
and it's important that these be accurate.
For example,
[The Rust Programming Book](https://doc.rust-lang.org/stable/book/) uses a lot
of code samples that could get outdated as the language evolves. Therefore it is very important for
them to be able to automatically test these code examples.

mdBook supports a `test` command that will run code samples as doc tests for your book. At
the moment, only Rust doc tests are supported.

#### Specify a directory

The `test` command can take a directory as an argument to use as the book's root
instead of the current working directory.

```bash
mdbook test path/to/book
```

#### `--dest-dir`

The `--dest-dir` (`-d`) option allows you to change the output directory for the
book. Relative paths are interpreted relative to the book's root directory. If
not specified it will default to the value of the `build.build-dir` key in
`book.toml`, or to `./book`.

#### `--chapter`

The `--chapter` (`-c`) option allows you to test a specific chapter of the
book using the chapter name or the relative path to the chapter.

#### `--library-path` `[`deprecated`]`

> Note: This argument doesn't provide sufficient extern crate information to run doc tests in current Rust compilers.  
Instead, add **manifest** to point to a **Cargo.toml** file in your ***book.toml***, as described in [rust configuration](/format/configuration/general.html#rust-options).


The `--library-path` (`-L`) option allows you to add directories to the library
search path used by `rustdoc` when it builds and tests the examples. Multiple
directories can be specified with multiple options (`-L foo -L bar`) or with a
comma-delimited list (`-L foo,bar`). The path should point to the Cargo
[build cache](https://doc.rust-lang.org/cargo/guide/build-cache.html) `deps` directory that
contains the build output of your project. For example, if your Rust project's book is in a directory
named `my-book`, the following command would include the crate's dependencies when running `test`:

```shell
mdbook test my-book -L target/debug/deps/
```

See the `rustdoc` command-line [documentation](https://doc.rust-lang.org/rustdoc/command-line-arguments.html#-l--library-path-where-to-look-for-dependencies)
for more information.
