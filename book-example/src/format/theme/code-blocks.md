# Code Blocks

Syntax highlighted code blocks are supported using [Highlight.js](https://highlightjs.org)
with a custom theme.  
Automatic language detection has been turned off, so you will probably want to
specify the programming language you use like this
<pre><code class="language-markdown">```rust
fn main() {
    // Some code
}
```</code></pre>

## Run Button
Rust code blocks are runnable by default with a run button, for example:
```rust
fn main() {
    println!("hello");
}
```
Other languages are currently not supported.

You can also diable the run button with `no_run`:
<pre><code class="language-markdown">```rust,no_run
fn main() {
    println!("Not runnable.");
}
```</code></pre>

```rust,no_run
fn main() {
    println!("Not runnable.");
}
```
### Show Warnings

It is possible to show warning when Rust code is run via adding `warn`:
<pre><code class="language-markdown">```rust,warn
fn main() {
    let unused = 7;
}
```</code></pre>

```rust,warn
fn main() {
    let unused = 7;
}
```

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

**At the moment, this only works for code examples that are annotated with
`rust`. Because it would collide with semantics of some programming languages.
In the future, we want to make this configurable through the `book.toml` so that
everyone can benefit from it.**


## Improve default theme

If you think the default theme doesn't look quite right for a specific language,
or could be improved. Feel free to [submit a new
issue](https://github.com/rust-lang/mdBook/issues) explaining what you
have in mind and I will take a look at it.

You could also create a pull-request with the proposed improvements.

Overall the theme should be light and sober, without to many flashy colors.
