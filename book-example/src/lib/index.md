# For Developers

While `mdbook` is mainly used as a command line tool, you can also import the 
underlying library directly and use that to manage a book. 

- Creating custom backends 
- Automatically generating and reloading a book on the fly 
- Integration with existing projects

The best source for examples on using the `mdbook` crate from your own Rust 
programs is the [API Docs].


## Alternate Backends

The mechanism for using alternative backends is very simple, you add an extra
table to your `book.toml` and the `MDBook::load()` function will detect the

For example, if you wanted to use a hypothetical `latex` backend you would add
an empty `output.latex` table to `book.toml`.

```toml
# book.toml

[book]
...

[output.latex]
``` 

And then during the rendering stage `mdbook` will run the `mdbook-latex`
program, piping it a JSON serialized [RenderContext] via stdin.

You can set the command used via the `command` key.

```toml
# book.toml

[book]
...

[output.latex]
command = "python3 my_plugin.py"
``` 

If no backend is supplied (i.e. there are no `output.*` tables), `mdbook` will 
fall back to the `html` backend.


[API Docs]: https://docs.rs/mdbook
[RenderContext]: https://docs.rs/mdbook/*/mdbook/renderer/struct.RenderContext.html