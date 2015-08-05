# Theme

The default renderer uses a [handlebars](http://handlebarsjs.com/) template to render your markdown files in and comes with a default theme
included in the mdBook binary.

But the theme is totally customizable, you can replace every file from the theme by your own by adding a
`theme` directory in your source folder. Create a new file with the name of the file you want to overwrite
and now that file will be used instead of the default file.

Here are the files you can overwrite:

- ***index.hbs*** is the handlebars template.
- ***book.css*** is the style used in the output. If you want to change the design of your book, this is probably the file you want to modify. Sometimes in conjunction with `index.hbs` when you want to radically change the layout.
- ***book.js*** is mostly used to add client side functionality.

**Note:**

When you overwrite a file, it is possible that you break some functionality. Therefore I recommend to use the file from the default theme as template and only add / modify what you need. In the future you will be able to copy the default theme into your source directory automatically by using `mdbook init --theme`.

### Syntax Highlighting

For syntax highlighting I use [Highlight.js](https://highlightjs.org) with modified theme.
But if you want a different theme, just put a `highlight.css` file in your theme folder and your theme will be used.

- ***highlight.js*** normally you shouldn't have to overwrite this file. But if you need to, you can.
- ***highlight.css*** theme used by highlight.js for syntax highlighting.

When write code blocks in your markdown you will probably want to specify the language you use

```markdown
```rust
```


## Handlebars

### Data

A lot of data is exposed to the handlebars template with the "context".
In the handlebars template you can access this information by using

```handlebars
{{name_of_property}}
```

Here is a list of the properties that are exposed:

- ***language*** Language of the book in the form `en`. To use in <code class="language-html">\<html lang="{{ language }}"></code> for example.
At the moment it is hardcoded.
- ***title*** Title of the book, as specified in `book.json`

- ***path*** Relative path to the original markdown file from the source directory
- ***content*** This is the rendered markdown.
- ***path_to_root*** This is a path containing exclusively `../`'s that points to the root of the book from the current file.
Since the original directory structure is maintained, it is useful to prepend relative links with this `path_to_root`.

- ***chapters*** Is an array of dictionaries of the form
  ```json
  {"section": "1.2.1", "name": "name of this chapter", "path": "dir/markdown.md"}
  ```
  containing all the chapters of the book. It is used for example to construct the table of contents (sidebar).

### Helpers

In addition to the properties you can access, there are some handlebars helpers at your disposal.

1.  ### toc

    The toc helper is used like this

    ```handlebars
    {{#toc}}{{/toc}}
    ```

    and outputs something that looks like this, depending on the structure of your book

    ```html
    <ul class="chapter">
        <li><a href="link/to/file.html">Some chapter</a></li>
        <li>
            <ul class="section">
                <li><a href="link/to/other_file.html">Some other Chapter</a></li>
            </ul>
        </li>
    </ul>
    ```

    If you would like to make a toc with another structure, you have access to the chapters property containing all the data.
    The only limitation at the moment is that you would have to do it with JavaScript instead of with a handlebars helper.

    ```html
    <script>
    var chapters = {{chapters}};
    // Processing here
    </script>
    ```

2.  ### previous / next

    The previous and next helpers expose a `link` and `name` property to the previous and next chapters.

    They are used like this

    ```handlebars
    {{#previous}}
        <a href="{{link}}" class="nav-chapters previous">
            <i class="fa fa-angle-left"></i>
        </a>
    {{/previous}}
    ```

    The inner html will only be rendered if the previous / next chapter exists.
    Of course the inner html can be changed to your liking.

------

If you would like me to expose other properties or helpers, please [create a new issue](https://github.com/azerupi/mdBook/issues)
and I will consider it.
