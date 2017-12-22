# For Developers

While `mdbook` is mainly used as a command line tool, you can also import the 
underlying library directly and use that to manage a book. 

- Creating custom backends 
- Automatically generating and reloading a book on the fly 
- Integration with existing projects

The best source for examples on using the `mdbook` crate from your own Rust 
programs is the [API Docs].


## Configuration

The mechanism for using alternative backends is very simple, you add an extra
table to your `book.toml` and the `MDBook::load()` function will automatically 
detect the backends being used.

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


## Render Context

The `RenderContext` encapsulates all the information a backend needs to know
in order to generate output. Its Rust definition looks something like this:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderContext {
    pub version: String,
    pub root: PathBuf,
    pub book: Book,
    pub config: Config,
    pub destination: PathBuf,
}
```

A backend will receive the `RenderContext` via `stdin` as one big JSON blob. If
possible, it is recommended to import the `mdbook` crate and use the 
`RenderContext::from_json()` method. This way you should always be able to 
deserialize the `RenderContext`, and as a bonus will also have access to the 
methods already defined on the underlying types.

Although backends are told the book's root directory on disk, it is *strongly
discouraged* to load chapter content from the filesystem. The `root` key is
provided as an escape hatch for certain plugins which may load additional,
non-markdown, files.


## Output Directory

To make things more deterministic, a backend will be told where it should place
its generated artefacts.

The general algorithm for deciding the output directory goes something like 
this:

- If there is only one backend:
    - `destination` is `config.build.build_dir` (usually `book/`)
- Otherwise:
    - `destination` is `config.build.build_dir` joined with the backend's name
      (e.g. `build/latex/` for the "latex" backend)


## Output and Signalling Failure

To signal that the plugin failed it just needs to exit with a non-zero return 
code. 

All output from the plugin's subprocess is immediately passed through to the
user, so it is encouraged for plugins to follow the ["rule of silence"] and
by default only tell the user about things they directly need to respond to
(e.g. an error in generation or a warning). 

This "silent by default" behaviour can be overridden via the `RUST_LOG`
environment variable (which `mdbook` will pass through to the backend if set)
as is typical with Rust applications.


[API Docs]: https://docs.rs/mdbook
[RenderContext]: https://docs.rs/mdbook/*/mdbook/renderer/struct.RenderContext.html
["rule of silence"]: http://www.linfo.org/rule_of_silence.html