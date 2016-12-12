# Configuration

You can configure the parameters for your book in the ***book.json*** file.

Here is an example of what a ***book.json*** file might look like:

```json
{
    "title": "Example book",
    "author": "Name",
    "description": "The example book covers examples.",
    "dest": "output/my-book"
}
```

#### Supported variables

If relative paths are given, they will be relative to the book's root, i.e. the
parent directory of the source directory.

- **title:** The title of the book.
- **author:** The author of the book.
- **description:** The description, which is added as meta in the html head of each page.
- **src:** The path to the book's source files (chapters in Markdown, SUMMARY.md, etc.). Defaults to `root/src`.
- **dest:** The path to the directory where you want your book to be rendered. Defaults to `root/book`.
- **theme_path:** The path to a custom theme directory. Defaults to `root/theme`.

***note:*** *the supported configurable parameters are scarce at the moment, but more will be added in the future*
