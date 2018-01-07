# Configuration

You can configure the parameters for your book in the ***book.toml*** file.

Here is an example of what a ***book.toml*** file might look like:

```toml
[book]
title = "Example book"
author = "John Doe"
description = "The example book covers examples."

[build]
build-dir = "my-example-book"
create-missing = false

[output.html]
additional-css = ["custom.css"]
```

## Supported configuration options

It is important to note that **any** relative path specified in the in the configuration will
always be taken relative from the root of the book where the configuration file is located.


### General metadata

This is general information about your book.

- **title:** The title of the book
- **authors:** The author(s) of the book
- **description:** A description for the book, which is added as meta
  information in the html `<head>` of each page
- **src:** By default, the source directory is found in the directory named
  `src` directly under the root folder. But this is configurable with the `src`
  key in the configuration file.

**book.toml**
```toml
[book]
title = "Example book"
authors = ["John Doe", "Jane Doe"]
description = "The example book covers examples."
src = "my-src"  # the source files will be found in `root/my-src` instead of `root/src`
```

### Build options

This controls the build process of your book.

- **build-dir:** The directory to put the rendered book in. By default this is
  `book/` in the book's root directory.
- **create-missing:** By default, any missing files specified in `SUMMARY.md`
  will be created when the book is built (i.e. `create-missing = true`). If this
  is `false` then the build process will instead exit with an error if any files
  do not exist.

**book.toml**
```toml
[build]
build-dir = "build"
create-missing = false
```

### HTML renderer options
The HTML renderer has a couple of options as well. All the options for the
renderer need to be specified under the TOML table `[output.html]`.

The following configuration options are available:

- **theme:** mdBook comes with a default theme and all the resource files
  needed for it. But if this option is set, mdBook will selectively overwrite
  the theme files with the ones found in the specified folder.
- **curly-quotes:** Convert straight quotes to curly quotes, except for
  those that occur in code blocks and code spans. Defaults to `false`.
- **google-analytics:** If you use Google Analytics, this option lets you
  enable it by simply specifying your ID in the configuration file.
- **additional-css:** If you need to slightly change the appearance of your
  book without overwriting the whole style, you can specify a set of
  stylesheets that will be loaded after the default ones where you can
  surgically change the style.
- **additional-js:** If you need to add some behaviour to your book without
  removing the current behaviour, you can specify a set of javascript files
  that will be loaded alongside the default one.
- **playpen:** A subtable for configuring various playpen settings.
- **no-section-label**: mdBook by defaults adds section label in table of
  contents column. For example, "1.", "2.1". Set this option to true to
  disable those labels. Defaults to `false`.

**book.toml**
```toml
[book]
title = "Example book"
authors = ["John Doe", "Jane Doe"]
description = "The example book covers examples."

[output.html]
theme = "my-theme"
curly-quotes = true
google-analytics = "123456"
additional-css = ["custom.css", "custom2.css"]
additional-js = ["custom.js"]

[output.html.playpen]
editor = "./path/to/editor"
editable = false
```
