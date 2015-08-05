# Syntax Highlighting

For syntax highlighting I use [Highlight.js](https://highlightjs.org) with a custom theme.

Automatic language detection has been turned off, so you will probably want to
specify the programming language you use like this

<pre class="language-markdown"><code class="language-markdown">```rust
fn main() {
    // Some code
}
```</code></pre>

## Improve default theme

If you think the default theme doesn't look quite right for a specific language, or could be improved.
Feel free to [submit a new issue](https://github.com/azerupi/mdBook/issues) explaining what you have in mind and I will take a look at it.

You could also create a pull-request with the proposed improvements.

Overall the theme should be light and sober, without to many flashy colors.


## Custom theme
Like the rest of the theme, the files used for syntax highlighting can be overwritten with your own.

- ***highlight.js*** normally you shouldn't have to overwrite this file. But if you need to, you can.
- ***highlight.css*** theme used by highlight.js for syntax highlighting.

If you want to use another theme for `highlight.js` download it from their website, or make it yourself,
rename it to `highlight.css` and put it in `src/theme` (or the equivalent if you changed your source folder)

Now your theme will be used instead of the default theme.
