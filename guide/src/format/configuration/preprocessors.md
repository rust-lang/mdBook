# Configuring Preprocessors

Preprocessors are extensions that can modify the raw Markdown source before it gets sent to the renderer.

The following preprocessors are built-in and included by default:

- `links`: Expands the `{{ #playground }}`, `{{ #include }}`, and `{{ #rustdoc_include }}` handlebars
  helpers in a chapter to include the contents of a file.
  See [Including files] for more.
- `index`: Convert all chapter files named `README.md` into `index.md`. That is
  to say, all `README.md` would be rendered to an index file `index.html` in the
  rendered book.

The built-in preprocessors can be disabled with the [`build.use-default-preprocessors`] config option.

The community has developed several preprocessors.
See the [Third Party Plugins] wiki page for a list of available preprocessors.

For information on how to create a new preprocessor, see the [Preprocessors for Developers] chapter.

[Including files]: ../mdbook.md#including-files
[`build.use-default-preprocessors`]: general.md#build-options
[Third Party Plugins]: https://github.com/rust-lang/mdBook/wiki/Third-party-plugins
[Preprocessors for Developers]: ../../for_developers/preprocessors.md

## Custom Preprocessor Configuration

Preprocessors can be added by including a `preprocessor` table in `book.toml` with the name of the preprocessor.
For example, if you have a preprocessor called `mdbook-example`, then you can include it with:

```toml
[preprocessor.example]
```

With this table, mdBook will execute the `mdbook-example` preprocessor.

This table can include additional key-value pairs that are specific to the preprocessor.
For example, if our example preprocessor needed some extra configuration options:

```toml
[preprocessor.example]
some-extra-feature = true
```

## Locking a Preprocessor dependency to a renderer

You can explicitly specify that a preprocessor should run for a renderer by
binding the two together.

```toml
[preprocessor.example]
renderers = ["html"]  # example preprocessor only runs with the HTML renderer
```

## Provide Your Own Command

By default when you add a `[preprocessor.foo]` table to your `book.toml` file,
`mdbook` will try to invoke the `mdbook-foo` executable. If you want to use a
different program name or pass in command-line arguments, this behaviour can
be overridden by adding a `command` field.

```toml
[preprocessor.random]
command = "python random.py"
```

## Require A Certain Order

The order in which preprocessors are run can be controlled with the `before` and `after` fields.
For example, suppose you want your `linenos` preprocessor to process lines that may have been `{{#include}}`d; then you want it to run after the built-in `links` preprocessor, which you can require using either the `before` or `after` field:

```toml
[preprocessor.linenos]
after = [ "links" ]
```

or

```toml
[preprocessor.links]
before = [ "linenos" ]
```

It would also be possible, though redundant, to specify both of the above in the same config file.

Preprocessors having the same priority specified through `before` and `after` are sorted by name.
Any infinite loops will be detected and produce an error.
