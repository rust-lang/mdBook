# Markdown tests

Tests for some markdown output.

## Tables

| foo | bar |
| --- | --- |
| baz | bim |

## Footnotes

Footnote example[^1], or with a word[^word].

[^1]: This is a footnote.

[^word]: A longer footnote.
    With multiple lines. [Link to unicode](unicode.md).
    With a reference inside.[^1]

There are multiple references to word[^word].

Footnote without a paragraph[^para]

[^para]:
    1. Item one
       1. Sub-item
    2. Item two

Footnote with multiple paragraphs[^multiple]

[^define-before-use]: This is defined before it is referred to.

<!-- Using <p> tags to work around rustdoc issue, this should move to a separate book.
     https://github.com/rust-lang/rust/issues/139064
-->
[^multiple]: <p>One</p><p>Two</p><p>Three</p>

[^unused]: This footnote is defined by not used.

Footnote name with wacky characters[^"wacky"]

[^"wacky"]: Testing footnote id with special characters.

Testing when referring to something earlier.[^define-before-use]

## Strikethrough

~~strikethrough example~~

## Tasklisks

- [X] Apples
- [X] Broccoli
- [ ] Carrots
