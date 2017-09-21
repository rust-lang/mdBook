# Configuration

You can configure the parameters for your book in the ***book.toml*** file.

>**Note:**  
JSON configuration files were previously supported but have been deprecated in favor of 
the TOML configuration file. If you are still using JSON we strongly encourage you to migrate to
the TOML configuration because JSON support will be removed in the future.

Here is an example of what a ***book.toml*** file might look like:

```toml
title = "Example book"
author = "John Doe"
description = "The example book covers examples."

[output.html]
destination = "my-example-book"
additional-css = ["my_custom.css"]
```

<div class="custom_border">Formatting elements with custom css.</div>

## Supported configuration options

It is important to note that **any** relative path specified in the in the configuration will
always be taken relative from the root of the book where the configuration file is located.

### General metadata

- **title:** The title of the book
- **author:** The author of the book
- **description:** A description for the book, which is added as meta information in the html `<head>` of each page

**book.toml**
```toml
title = "Example book"
author = "John Doe"
description = "The example book covers examples."
```

Some books may have multiple authors, there is an alternative key called `authors` plural that lets you specify an array
of authors.

**book.toml**
```toml
title = "Example book"
authors = ["John Doe", "Jane Doe"]
description = "The example book covers examples."
```

### Source directory
By default, the source directory is found in the directory named `src` directly under the root folder. But this is configurable
with the `source` key in the configuration file.

**book.toml**
```toml
title = "Example book"
authors = ["John Doe", "Jane Doe"]
description = "The example book covers examples."

source = "my-src"  # the source files will be found in `root/my-src` instead of `root/src`
```

### HTML renderer options
The HTML renderer has a couple of options aswell. All the options for the renderer need to be specified under the TOML table `[output.html]`.
The following configuration options are available:

- **`destination`:** By default, the HTML book will be rendered in the `root/book` directory, but this option lets you specify another
  destination fodler.
- **`theme`:** mdBook comes with a default theme and all the resource files needed for it. But if this option is set, mdBook will selectively overwrite the theme files with the ones found in the specified folder. 
- **`curly-quotes`:** Convert straight quotes to curly quotes, except for those that occur in code blocks and code spans. Defaults to `false`.
- **`google-analytics`:** If you use Google Analytics, this option lets you enable it by simply specifying your ID in the configuration file.
- **`additional-css`:** If you need to slightly change the appearance of your book without overwriting the whole style, you can specify a set of stylesheets that will be loaded after the default ones where you can surgically change the style.
- **`additional-js`:** If you need to add some behaviour to your book without removing the current behaviour, you can specify a set of javascript files that will be loaded alongside the default one.

**book.toml**
```toml
title = "Example book"
authors = ["John Doe", "Jane Doe"]
description = "The example book covers examples."

[output.html]
destination = "my-book" # the output files will be generated in `root/my-book` instead of `root/book`
theme = "my-theme"
curly-quotes = true
google-analytics = "123456"
additional-css = ["custom.css", "custom2.css"]
additional-js = ["custom.js"]
```

