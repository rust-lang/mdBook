# MathJax support

mdBook has optional support for math formulas, which are rendered through
[MathJax](https://www.mathjax.org/) (more precisely, the latest release of
MathJax version 4). To enable it, set `output.html.math` to `true` in your
`book.toml`:

```toml
[output.html]
math = true
```

Inline equations are delimited by `$...$` and block equations are delimited
by `$$...$$`. For example, to obtain

> If $n \geq 3$ then there are no integers $a, b, c \geq 1$ satisfying $$a^n + b^n = c^n$$

you would write the following:

```
If $n \geq 3$ then there are no integers $a, b, c \geq 1$ satisfying $$a^n + b^n = c^n$$
```

## Legacy MathJax support

The legacy option `output.html.mathjax-support` enables equations with a
different syntax: `\\( ... \\)` for inline equations and `\\[ ... \\]` for
block equations. Because it does not parse formulas in the Markdown input
but instead lets MathJax find the delimiters in the HTML output, it has
the limitation that characters which have an effect in Markdown, such as
underscores, need to be escaped in formulas. This option is kept for
backwards compatibility; use `output.html.math` in new books. It uses
MathJax 2.7.1.
