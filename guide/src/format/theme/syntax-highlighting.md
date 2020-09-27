# Syntax Highlighting

For syntax highlighting I use [Highlight.js](https://highlightjs.org) with a
custom theme.

Automatic language detection has been turned off, so you will probably want to
specify the programming language you use like this

<pre><code class="language-markdown">```rust
fn main() {
    // Some code
}
```</code></pre>

## Custom theme
Like the rest of the theme, the files used for syntax highlighting can be
overridden with your own.

- ***highlight.js*** normally you shouldn't have to overwrite this file, unless
  you want to use a more recent version.
- ***highlight.css*** theme used by highlight.js for syntax highlighting.

If you want to use another theme for `highlight.js` download it from their
website, or make it yourself, rename it to `highlight.css` and put it in
the `theme` folder of your book.

Now your theme will be used instead of the default theme.

## Hiding code lines

There is a feature in mdBook that lets you hide code lines by prepending them
with a `#`.


```bash
# fn main() {
    let x = 5;
    let y = 6;

    println!("{}", x + y);
# }
```

Will render as

```rust
# fn main() {
    let x = 5;
    let y = 7;

    println!("{}", x + y);
# }
```

By default, this only works for code examples that are annotated with `rust`. However, you can 
define custom patterns for other languages in your `book.toml`. Unless you need something complex 
(e.g. rust uses `#` but doesn't hide `#[...]` lines), adding a new language is trivial. Just add 
a new `boring-prefix` entry in your `book.toml` with the language name and prefix character 
(you can also do multi-character prefixes if you really want to):

```toml
[output.html.playground.boring-prefixes]
python = "~"
```

The auto-generated prefix patterns will hide any lines that begin with the given prefix, but
the prefix can be escaped using a backslash. If you need something more complex than that, 
you can use a [fully custom pattern](../config.md#boring-patterns).

## Improve default theme

If you think the default theme doesn't look quite right for a specific language,
or could be improved. Feel free to [submit a new
issue](https://github.com/rust-lang/mdBook/issues) explaining what you
have in mind and I will take a look at it.

You could also create a pull-request with the proposed improvements.

Overall the theme should be light and sober, without to many flashy colors.
