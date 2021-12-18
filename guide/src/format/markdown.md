# Markdown

mdBook's [parser](https://github.com/raphlinus/pulldown-cmark) adheres to the [CommonMark](https://commonmark.org/)
specification. You can take a quick [tutorial](https://commonmark.org/help/tutorial/),
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

## Lists
