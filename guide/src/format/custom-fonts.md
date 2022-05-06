# Custom fonts

It is possible to use custom fonts by using a combination of the parameters
`copy-fonts` and `copy-custom-fonts`. `copy-fonts` is enabled by default.

The fonts are specified by the `fonts/fonts.css` file, relative to the root directory
of your book project.

## The copy-fonts option

If `copy-fonts` is enabled, the build in `fonts/fonts.css` file is being copied
to the output directory. In addition to that, the default fonts are copied to
the output directory.

## The copy-custom-fonts option

If `copy-custom-fonts` is enabled, the `fonts/fonts.css` file that you have created
is copied to the output directory. Also, the file gets parsed and searched for
`@font-face` rules having a `src: url(CUSTOM-FONT)` definition. These `CUSTOM-FONT`
files are copied to the output directory.

## Enabling both options

It is also possible to use both options in parallel. Then, the build in fonts
are copied, as well as the custom font files that have been found. The build in
and the custom definition of `fonts/fonts.css` are combined and copied to the
output directory as well.

## Example with both options enabled

Create a `fonts/` folder in your book root. Then copy the fonts you want to use
into this folder (in this example we are assuming `Lato-Regular.ttf`)

Create a custom fonts file `fonts/fonts.css`

```css
@font-face {
     font-family: "Lato";
     font-style: normal;
     font-weight: normal;
     src: url('Lato-Regular.ttf');
}
```

Setup your `book.toml` that it contains the following parameters:

```toml
[output.html]
copy-fonts = true
copy-custom-fonts = true
```

Adjust your `theme/css/general.css` file according to your needs, for example:

```css
html {
    font-family: "Lato", sans-serif;
}
```
