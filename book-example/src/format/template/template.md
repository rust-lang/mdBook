# Template

The default renderer uses a [handlebars](http://handlebarsjs.com/) template to
render your markdown files and comes with a default theme included in the mdBook
binary.

If you wish to customise this, call `mdbook init --copy-assets` to get a copy of
the default template and its assets.

The page template to render the HTML pages is expected at `assets/_html-template/_layouts/page.hbs`.

Everything else can be changed, since it only depends on the paths you write in `page.hbs`.

The stylesheets are compiled with [stylus](http://stylus-lang.com/). To use
this, install both `stylus` and the `nib` helpers:

`npm install stylus nib -g`

