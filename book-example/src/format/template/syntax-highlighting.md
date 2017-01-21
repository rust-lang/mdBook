# Syntax Highlighting

For syntax highlighting I use [Highlight.js](https://highlightjs.org) with a custom theme.

Automatic language detection has been turned off, so you will probably want to
specify the programming language you use like this

<pre><code class="language-markdown">```rust
fn main() {
    // Some code
}
```</code></pre>

## Custom theme

To customise its CSS, call `mdbook init --copy-assets` to get the default static assets including the stylus files. Edit these and regenerate `book.css`:

```
stylus book.styl -o ../css/book.css --use nib
```

## Hiding code lines

There is a feature in mdBook that let's you hide code lines by prepending them with a `#`.


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

**At the moment, this only works for code examples that are annotated with `rust`. Because it would collide with semantics of some programming languages. In the future, we want to make this configurable through the `book.toml` so that everyone can benefit from it.**


## Improve default theme

If you think the default theme doesn't look quite right for a specific language, or could be improved.
Feel free to [submit a new issue](https://github.com/azerupi/mdBook/issues) explaining what you have in mind and I will take a look at it.

You could also create a pull-request with the proposed improvements.

Overall the theme should be light and sober, without to many flashy colors.
