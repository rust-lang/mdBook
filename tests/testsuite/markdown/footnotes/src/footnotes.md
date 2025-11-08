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

Footnote that is defined multiple times.[^multiple-definitions]

[^multiple-definitions]: This is the first definition of the footnote with tag multiple-definitions

And another[^in-between] that references the duplicate again.[^multiple-definitions]

[^in-between]: Footnote between duplicates.

[^multiple-definitions]: This is the second definition of the footnote with tag multiple-definitions

Multiple footnotes in a row.[^a][^b][^c]

[^a]: Footnote 1
[^b]: Footnote 2
[^c]: Footnote 3
