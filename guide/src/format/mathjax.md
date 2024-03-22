# MathJax Support

mdBook has optional support for math equations through
[MathJax](https://www.mathjax.org/).

>**Note:** The usual delimiters MathJax uses are not yet supported. You can't
currently use `$$ ... $$` as delimiters and the `\[ ... \]` delimiters need an
extra backslash to work. Hopefully this limitation will be lifted soon.

>**Note:** When you use double backslashes in MathJax blocks (for example in
> commands such as `\begin{cases} \frac 1 2 \\ \frac 3 4 \end{cases}`) you need
> to add _two extra_ backslashes (e.g., `\begin{cases} \frac 1 2 \\\\ \frac 3 4
> \end{cases}`).


### Inline equations
Inline equations are delimited by `\\(` and `\\)`. So for example, to render the
following inline equation \\( \int x dx = \frac{x^2}{2} + C \\) you would write
the following:
```
\\( \int x dx = \frac{x^2}{2} + C \\)
```

### Block equations
Block equations are delimited by `\\[` and `\\]`. To render the following
equation

\\[ \mu = \frac{1}{N} \sum_{i=0} x_i \\]


you would write:

```bash
\\[ \mu = \frac{1}{N} \sum_{i=0} x_i \\]
```

## MathJax 2

To enable MathJax 2, you need to add the `mathjax-support` key to your `book.toml`
under the `output.html` section.

```toml
[output.html]
mathjax-support = true
```

## MathJax 3

To enable MathJax 3, you need to add the `enable` key to your `book.toml`
under the `output.html.mathjax` section.

```toml
[output.html.mathjax]
enable = true
```
>**Note:** Remove or set to the `false` value the `mathjax-support` key
> under the `output.html` section if you set it previously.

Additionaly you can set `config` key to select used [configuration][comb-comp].
You can select one of:

| Value | Input | Output | Default
| :-- | :-- | :-- | :--
| tex-chtml | tex | chtml |
| tex-chtml-full | tex | chtml |
| tex-svg | tex | svg |
| tex-svg-full | tex | svg |
| tex-mml-chtml | tex, mml | chtml | yes
| tex-mml-svg | tex, mml | svg |
| mml-chtml | mml | chtml |
| mml-svg | mml | svg |

Use the `source` key to set used MathJax distribution.
If value starts with `/` symbol it will be interpreted relative
to [source/build](configuration/general.md) directory.
By default the builtin copy of MathJax with `/mathjax/es5` value is used.
But you can set some value like the `https://cdn.jsdelivr.net/npm/mathjax@3/es5`
to use MathJax from CDN.

For example to use local MathJax copy for tex input and svg output you can
do something like:

```console
$ wget https://github.com/mathjax/MathJax/archive/refs/tags/3.2.2.tar.gz
$ tar -xf 3.2.2.tar.gz MathJax-3.2.2/es5
$ mv MathJax-3.2.2 src/mathjax@3
```

Then add to your `book.toml` file:

```toml
[output.html.mathjax]
enable = true
source = "/mathjax@3/es5"
config = "tex-svg"
```

[comb-comp]: https://docs.mathjax.org/en/v3.2-latest/web/components/combined.html
