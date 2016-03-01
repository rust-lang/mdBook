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

- **title:** title of the book
- **author:** author of the book
- **description:** description, which is added as meta in the html head of each page.
- **dest:** path to the directory where you want your book to be rendered. If a relative path is given it will be relative to the parent directory of the source directory

***note:*** *the supported configurable parameters are scarce at the moment, but more will be added in the future*
