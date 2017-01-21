# Configuration

You can configure the parameters for your book in the ***book.toml*** file.

We encourage using the TOML format, but JSON is also recognized and parsed.

Here is an example of what a ***book.toml*** file might look like:

```toml
title = "Example book"
author = "Name"
description = "The example book covers examples."
dest_base = "output/my-book"
```

#### Supported variables

If relative paths are given, they will be relative to the book's root, i.e. the
parent directory of the source directory.

- **title:** The title of the book.
- **author:** The author of the book.
- **description:** The description, which is added as meta in the html head of each page.
- **src_base:** The path to the book's source files (chapters in Markdown, SUMMARY.md, etc.). Defaults to `src/`.
- **dest_base:** The path to the directory where you want your book to be rendered. Defaults to `book/`.

***note:*** *the supported configurable parameters are scarce at the moment, but more will be added in the future*
