# Footnote tests

Footnote example[^1], or with a word[^word].

[^1]: This is a footnote.

[^word]: A longer footnote.
    With multiple lines. [Link to other](other.md).
    With a reference inside.[^1]

There are multiple references to word[^word].

Footnote without a paragraph[^para]

[^para]:
    1. Item one
       1. Sub-item
    2. Item two

Footnote with multiple paragraphs[^multiple]

[^define-before-use]: This is defined before it is referred to.

[^multiple]:
    One

    Two

    Three

[^unused]: This footnote is defined by not used.

Footnote name with wacky characters[^"wacky"]

[^"wacky"]: Testing footnote id with special characters.

Testing when referring to something earlier.[^define-before-use]
