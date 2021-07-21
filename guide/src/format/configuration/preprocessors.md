# Configuring Preprocessors

The following preprocessors are available and included by default:

- `links`: Expand the `{{ #playground }}`, `{{ #include }}`, and `{{ #rustdoc_include }}` handlebars
  helpers in a chapter to include the contents of a file.
- `index`: Convert all chapter files named `README.md` into `index.md`. That is
  to say, all `README.md` would be rendered to an index file `index.html` in the
  rendered book.


**book.toml**
```toml
[build]
build-dir = "build"
create-missing = false

[preprocessor.links]

[preprocessor.index]
```

### Custom Preprocessor Configuration

Like renderers, preprocessor will need to be given its own table (e.g.
`[preprocessor.mathjax]`). In the section, you may then pass extra
configuration to the preprocessor by adding key-value pairs to the table.

For example

```toml
[preprocessor.links]
# set the renderers this preprocessor will run for
renderers = ["html"]
some_extra_feature = true
```

#### Locking a Preprocessor dependency to a renderer

You can explicitly specify that a preprocessor should run for a renderer by
binding the two together.

```toml
[preprocessor.mathjax]
renderers = ["html"]  # mathjax only makes sense with the HTML renderer
```

### Provide Your Own Command

By default when you add a `[preprocessor.foo]` table to your `book.toml` file,
`mdbook` will try to invoke the `mdbook-foo` executable. If you want to use a
different program name or pass in command-line arguments, this behaviour can
be overridden by adding a `command` field.

```toml
[preprocessor.random]
command = "python random.py"
```

### Require A Certain Order

The order in which preprocessors are run is not guaranteed, but you can request some to run before or after others.
For example, suppose you want your `linenos` preprocessor to process lines that may have been `{{#include}}`d; then you want it to run after the built-in `links` preprocessor, which you can require using the `before` and/or `after` fields.

```toml
[preprocessor.linenos]
after = [ "links" ]
```

or

```toml
[preprocessor.links]
before = [ "linenos" ]
```

It would be possible, though redundant, to specify both of the above in the same config file.

`mdbook` will detect any infinite loops and error out.
Note that order of preprocessors besides what is specified using `before` and `after` is not guaranteed, and may change within the same run of `mdbook`.