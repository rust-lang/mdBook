# SUMMARY.md

The summary file is used by mdBook to know what chapters to include,
in what order they should appear, what their hierarchy is and where the source files are.
Without this file, there is no book.

Even though `SUMMARY.md` is a markdown file, the formatting is very strict to
allow for easy parsing. Let's see how you should format your `SUMMARY.md` file.

Markdown elements that are not listed below, are unsupported in this file and
will be ignored at best or result in an error.

A simple `SUMMARY.md` might look like this:

```markdown
# Title

[Introduction](introduction.md)

- [First Chapter](first-chapter.md)
- [Second Chapter]()

[Glossary](glossary.md)
```

#### Allowed elements

***Title***

It's common practice to begin with a title, generally <code
class="language-markdown"># Summary</code>. But it is not mandatory, the parser
just ignores it. So you can too if you feel like it.

***Frontmatter Chapters***

Before the main numbered chapters you can add a couple of elements that will not
be numbered. This is useful for forewords, introductions, etc.

There are however some constraints. You can not nest unnunmbered chapters, they
should all be on the root level. And you can not add unnunmbered chapters once
you have added numbered chapters.

```markdown
[Title of prefix element](relative/path/to/markdown.md)
```

***Mainmatter Chapters***

Numbered chapters are the main content of the book, they will be numbered and
can be nested, resulting in a nice hierarchy (chapters, sub-chapters, etc.)
   
```markdown
- [Title of the Chapter](relative/path/to/markdown.md)
```

You can either use `-` or `*` to indicate a numbered chapter.

***Backmatter Chapters***

After the numbered chapters you can add unnumbered chapters.

