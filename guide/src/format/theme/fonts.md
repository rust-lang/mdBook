# Custom fonts

Since v0.5, fonts are considered as static assets and thus are fingerprinted.
[This brought confusion](https://github.com/rust-lang/mdBook/issues/2958) about how to reference them from CSS files.
This is an example showing how to do so:

As a reminder, all fonts living under `./theme/fonts/` will be considered static and will be fingerprinted.  
Also, according to [the theme documentation](index.html#theme), additional fonts can be declared in `./theme/fonts/fonts.css`, which is also considered static.

That allows us to leverage the [`resource` helper](index-hbs.html#resource) inside `/theme/fonts/fonts.css`:

```css
@font-face {
    font-family: 'custom-font';
    src: url('{{ resource "fonts/Roboto.ttf" }}') format('truetype');
}
```

The `{{ resource "fonts/Roboto.ttf" }}` will be replaced by the fingerprinted name of the file, and now any HTML block with `font-family: custom-font` will have a Roboto font.

In my case:

```bash
$ ls book/fonts/
fonts-f9156add.css  Roboto-27bccdc7.ttf
```
