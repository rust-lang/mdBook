dark.tmTheme is from https://github.com/chriskempson/base16-textmate/blob/master/Themes/base16-tomorrow-night.tmTheme

ayu.tmTheme is a tweaked version of the dark theme

light.tmTheme is from https://github.com/atelierbram/syntax-highlighting/blob/master/docs/archive/atelier-schemes/output/textmate/base16-atelierdune.light.tmTheme which is described at https://atelierbram.github.io/syntax-highlighting/atelier-schemes/dune/

This folder is not copied over to the book directory when using `mdbook init`, nor is it indexed at runtime. To modify the themes, modify the tmTheme file, then run:

```shell
$ cargo run -- gen-syntax-cache --themes-only --no-default-themes --dest-dir=..
```

The `--no-default-themes` flag is added because the whole point is to *not* use the ones embedded in the mdbook binary.

Now, you may rebuild the `mdbook` binary and the new themes should be included.

This only needs to be run when the themes in this folder are updated.
