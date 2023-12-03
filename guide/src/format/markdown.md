# Markdown

mdBook's [parser](https://github.com/raphlinus/pulldown-cmark) adheres to the [CommonMark](https://commonmark.org/) specification with some extensions described below.
You can take a quick [tutorial](https://commonmark.org/help/tutorial/),
or [try out](https://spec.commonmark.org/dingus/) CommonMark in real time. A complete Markdown overview is out of scope for 
this documentation, but below is a high level overview of some of the basics. For a more in-depth experience, check out the
[Markdown Guide](https://www.markdownguide.org).

## Text and Paragraphs

Text is rendered relatively predictably: 

```markdown
Here is a line of text.

This is a new line.
```

Will look like you might expect:

Here is a line of text.

This is a new line.

## Headings

Headings use the `#` marker and should be on a line by themselves. More `#` mean smaller headings:

```markdown
### A heading 

Some text.

#### A smaller heading 

More text.
```

### A heading 

Some text.

#### A smaller heading 

More text.

## Lists

Lists can be unordered or ordered. Ordered lists will order automatically:

```markdown
* milk
* eggs
* butter

1. carrots
1. celery
1. radishes
```

* milk
* eggs
* butter

1. carrots
1. celery
1. radishes

## Links

Linking to a URL or local file is easy:

```markdown
Use [mdBook](https://github.com/rust-lang/mdBook). 

Read about [mdBook](mdbook.md).

A bare url: <https://www.rust-lang.org>.
```

Use [mdBook](https://github.com/rust-lang/mdBook). 

Read about [mdBook](mdbook.md).

A bare url: <https://www.rust-lang.org>.

----

Relative links that end with `.md` will be converted to the `.html` extension.
It is recommended to use `.md` links when possible.
This is useful when viewing the Markdown file outside of mdBook, for example on GitHub or GitLab which render Markdown automatically.

Links to `README.md` will be converted to `index.html`.
This is done since some services like GitHub render README files automatically, but web servers typically expect the root file to be called `index.html`.

You can link to individual headings with `#` fragments.
For example, `mdbook.md#text-and-paragraphs` would link to the [Text and Paragraphs](#text-and-paragraphs) section above.
The ID is created by transforming the heading such as converting to lowercase and replacing spaces with dashes.
You can click on any heading and look at the URL in your browser to see what the fragment looks like.

## Images

Including images is simply a matter of including a link to them, much like in the _Links_ section above. The following markdown
includes the Rust logo SVG image found in the `images` directory at the same level as this file:

```markdown
![The Rust Logo](images/rust-logo-blk.svg)
```

Produces the following HTML when built with mdBook:

```html
<p><img src="images/rust-logo-blk.svg" alt="The Rust Logo" /></p>
```

Which, of course displays the image like so:

![The Rust Logo](images/rust-logo-blk.svg)

## Extensions

mdBook has several extensions beyond the standard CommonMark specification.

### Strikethrough

Text may be rendered with a horizontal line through the center by wrapping the
text with one or two tilde characters on each side:

```text
An example of ~~strikethrough text~~.
```

This example will render as:

> An example of ~~strikethrough text~~.

This follows the [GitHub Strikethrough extension][strikethrough].

### Footnotes

A footnote generates a small numbered link in the text which when clicked
takes the reader to the footnote text at the bottom of the item. The footnote
label is written similarly to a link reference with a caret at the front. The
footnote text is written like a link reference definition, with the text
following the label. Example:

```text
This is an example of a footnote[^note].

[^note]: This text is the contents of the footnote, which will be rendered
    towards the bottom.
```

This example will render as:

> This is an example of a footnote[^note].
>
> [^note]: This text is the contents of the footnote, which will be rendered
>     towards the bottom.

The footnotes are automatically numbered based on the order the footnotes are
written.

### Tables

Tables can be written using pipes and dashes to draw the rows and columns of
the table. These will be translated to HTML table matching the shape. Example:

```text
| Header1 | Header2 |
|---------|---------|
| abc     | def     |
```

This example will render similarly to this:

| Header1 | Header2 |
|---------|---------|
| abc     | def     |

See the specification for the [GitHub Tables extension][tables] for more
details on the exact syntax supported.

### Task lists

Task lists can be used as a checklist of items that have been completed.
Example:

```md
- [x] Complete task
- [ ] Incomplete task
```

This will render as:

> - [x] Complete task
> - [ ] Incomplete task

See the specification for the [task list extension] for more details.

### Smart punctuation

Some ASCII punctuation sequences will be automatically turned into fancy Unicode
characters:

| ASCII sequence | Unicode |
|----------------|---------|
| `--`           | –       |
| `---`          | —       |
| `...`          | …       |
| `"`            | “ or ”, depending on context |
| `'`            | ‘ or ’, depending on context |

So, no need to manually enter those Unicode characters!

This feature is disabled by default.
To enable it, see the [`output.html.curly-quotes`] config option.

[strikethrough]: https://github.github.com/gfm/#strikethrough-extension-
[tables]: https://github.github.com/gfm/#tables-extension-
[task list extension]: https://github.github.com/gfm/#task-list-items-extension-
[`output.html.curly-quotes`]: configuration/renderers.md#html-renderer-options

### Heading attributes

Headings can have a custom HTML ID and classes. This lets you maintain the same ID even if you change the heading's text, it also lets you add multiple classes in the heading.

Example:
```md
# Example heading { #first .class1 .class2 }
```

This makes the level 1 heading with the content `Example heading`, ID `first`, and classes `class1` and `class2`. Note that the attributes should be space-separated.

More information can be found in the [heading attrs spec page](https://github.com/raphlinus/pulldown-cmark/blob/master/specs/heading_attrs.txt).
