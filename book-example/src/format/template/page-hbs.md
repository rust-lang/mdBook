# page.hbs

`page.hbs` is the handlebars template that is used to render the book.
The markdown files are processed to html and then injected in that template.

If you want to change the layout or style of your book, chances are that you will
have to modify this template a little bit. Here is what you need to know.

## Data

A lot of data is exposed to the handlebars template with the "context".
In the handlebars template you can access this information by using

```handlebars
{{name_of_property}}
```

Here is a list of the properties that are exposed:

- ***language*** Language of the book in the form `en`. To use in <code class="language-html">\<html lang="{{ language }}"></code> for example. This is `en` by default, or the language key of the translation as given in `book.toml`.
- ***title*** Title of the book, as specified in `book.toml`
- ***page_title*** A page title that includes the chapter title, used for the `<title>` tag
- ***path*** Relative path to the original markdown file from the source directory
- ***content*** This is the rendered markdown.
- ***path_to_root*** This is a path containing exclusively `../`'s that points to the root of the book from the current file.

A `<base>` tag is inserted in the template which uses `{{path_to_root}}` to
maintain correct paths to relative links.

- ***chapters*** Is an array of dictionaries of the form
  ```json
  {"section": "1.2.1", "name": "name of this chapter", "path": "dir/markdown.md"}
  ```
  containing all the chapters of the book. It is used for example to construct the table of contents (sidebar).

## Handlebars Helpers

In addition to the properties you can access, there are some handlebars helpers at your disposal.

### toc

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

If you would like to make a toc with another structure, you have access to the
chapters property containing all the data. The only limitation at the moment is
that you would have to do it with JavaScript instead of with a handlebars
helper.

```html
<script>
var chapters = {{chapters}};
// Processing here
</script>
```

### previous / next

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

### translation_indexes

Adds links to the index page of each translation.

### translation_links

Adds chapter-to-chapter links between translations.

### customcss

If a `custom.css` file is found, it adds a `<link>` tag to it.

------

*If you would like me to expose other properties or helpers, please [create a new issue](https://github.com/azerupi/mdBook/issues)
and I will consider it.*
