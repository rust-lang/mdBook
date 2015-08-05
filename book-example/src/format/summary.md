# SUMMARY.md

The summary file is used by mdBook to know what chapters to include,
in what order they should appear, what their hierarchy is and where the source files are.
Without this file, there is no book.

Even though `SUMMARY.md` is a markdown file, the formatting is very strict to
allow for easy parsing. Let's see how you should format your `SUMMARY.md` file.

#### Allowed elements

1. ***Title*** It's common practice to begin with a title, generally
   <code class="language-markdown"># Summary</code>.
   But it is not mandatory, the parser just ignores it. So you can too
   if you feel like it.

2. ***list link*** the other elements have to be list elements in form of a link
   ```markdown
   - [Title of the Chapter](relative/path/to/markdown.md)
   ```
   You can either use `-` or `*` to indicate a list. The lists can be nested,
   resulting in a nice hierarchy (chapters, sub-chapters, etc.)

All other elements are unsupported and will be ignored at best or result in an error.

#### not yet implemented

In the feature I would like to add support for links without the need to be list elements
at the root level to add chapters that don't need numbering, like an index, appendix,
contributor list, introduction, foreword, etc.
