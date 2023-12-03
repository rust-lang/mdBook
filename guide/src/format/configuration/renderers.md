# Configuring Renderers

Renderers (also called "backends") are responsible for creating the output of the book.

The following backends are built-in:

* [`html`](#html-renderer-options) — This renders the book to HTML.
  This is enabled by default if no other `[output]` tables are defined in `book.toml`.
* [`markdown`](#markdown-renderer) — This outputs the book as markdown after running the preprocessors.
  This is useful for debugging preprocessors.

The community has developed several backends.
See the [Third Party Plugins] wiki page for a list of available backends.

For information on how to create a new backend, see the [Backends for Developers] chapter.

[Third Party Plugins]: https://github.com/rust-lang/mdBook/wiki/Third-party-plugins
[Backends for Developers]: ../../for_developers/backends.md

## Output tables

Backends can be added by including a `output` table in `book.toml` with the name of the backend.
For example, if you have a backend called `mdbook-wordcount`, then you can include it with:

```toml
[output.wordcount]
```

With this table, mdBook will execute the `mdbook-wordcount` backend.

This table can include additional key-value pairs that are specific to the backend.
For example, if our example backend needed some extra configuration options:

```toml
[output.wordcount]
ignores = ["Example Chapter"]
```

If you define any `[output]` tables, then the `html` backend is not enabled by default.
If you want to keep the `html` backend running, then just include it in the `book.toml` file.
For example:

```toml
[book]
title = "My Awesome Book"

[output.wordcount]

[output.html]
```

If more than one `output` table is included, this changes the behavior for the layout of the output directory.
If there is only one backend, then it places its output directly in the `book` directory (see [`build.build-dir`] to override this location).
If there is more than one backend, then each backend is placed in a separate directory underneath `book`.
For example, the above would have directories `book/html` and `book/wordcount`.

[`build.build-dir`]: general.md#build-options

### Custom backend commands

By default when you add an `[output.foo]` table to your `book.toml` file,
`mdbook` will try to invoke the `mdbook-foo` executable.
If you want to use a different program name or pass in command-line arguments,
this behaviour can be overridden by adding a `command` field.

```toml
[output.random]
command = "python random.py"
```

### Optional backends

If you enable a backend that isn't installed, the default behavior is to throw an error.
This behavior can be changed by marking the backend as optional:

```toml
[output.wordcount]
optional = true
```

This demotes the error to a warning.


## HTML renderer options

The HTML renderer has a variety of options detailed below.
They should be specified in the `[output.html]` table of the `book.toml` file.

```toml
# Example book.toml file with all output options.
[book]
title = "Example book"
authors = ["John Doe", "Jane Doe"]
description = "The example book covers examples."

[output.html]
theme = "my-theme"
default-theme = "light"
preferred-dark-theme = "navy"
curly-quotes = true
mathjax-support = false
copy-fonts = true
additional-css = ["custom.css", "custom2.css"]
additional-js = ["custom.js"]
no-section-label = false
git-repository-url = "https://github.com/rust-lang/mdBook"
git-repository-icon = "fa-github"
edit-url-template = "https://github.com/rust-lang/mdBook/edit/master/guide/{path}"
site-url = "/example-book/"
cname = "myproject.rs"
input-404 = "not-found.md"
```

The following configuration options are available:

- **theme:** mdBook comes with a default theme and all the resource files needed
  for it. But if this option is set, mdBook will selectively overwrite the theme
  files with the ones found in the specified folder.
- **default-theme:** The theme color scheme to select by default in the
  'Change Theme' dropdown. Defaults to `light`.
- **preferred-dark-theme:** The default dark theme. This theme will be used if
  the browser requests the dark version of the site via the
  ['prefers-color-scheme'](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme)
  CSS media query. Defaults to `navy`.
- **curly-quotes:** Convert straight quotes to curly quotes, except for those
  that occur in code blocks and code spans. Defaults to `false`.
- **mathjax-support:** Adds support for [MathJax](../mathjax.md). Defaults to
  `false`.
- **copy-fonts:** (**Deprecated**) If `true` (the default), mdBook uses its built-in fonts which are copied to the output directory.
  If `false`, the built-in fonts will not be used.
  This option is deprecated. If you want to define your own custom fonts,
  create a `theme/fonts/fonts.css` file and store the fonts in the `theme/fonts/` directory.
- **google-analytics:** This field has been deprecated and will be removed in a future release.
  Use the `theme/head.hbs` file to add the appropriate Google Analytics code instead.
- **additional-css:** If you need to slightly change the appearance of your book
  without overwriting the whole style, you can specify a set of stylesheets that
  will be loaded after the default ones where you can surgically change the
  style.
- **additional-js:** If you need to add some behaviour to your book without
  removing the current behaviour, you can specify a set of JavaScript files that
  will be loaded alongside the default one.
- **no-section-label:** mdBook by defaults adds numeric section labels in the table of
  contents column. For example, "1.", "2.1". Set this option to true to disable
  those labels. Defaults to `false`.
- **git-repository-url:**  A url to the git repository for the book. If provided
  an icon link will be output in the menu bar of the book.
- **git-repository-icon:** The FontAwesome icon class to use for the git
  repository link. Defaults to `fa-github` which looks like <i class="fa fa-github"></i>.
  If you are not using GitHub, another option to consider is `fa-code-fork` which looks like <i class="fa fa-code-fork"></i>.
- **edit-url-template:** Edit url template, when provided shows a
  "Suggest an edit" button (which looks like <i class="fa fa-edit"></i>) for directly jumping to editing the currently
  viewed page. For e.g. GitHub projects set this to
  `https://github.com/<owner>/<repo>/edit/<branch>/{path}` or for
  Bitbucket projects set it to
  `https://bitbucket.org/<owner>/<repo>/src/<branch>/{path}?mode=edit`
  where {path} will be replaced with the full path of the file in the
  repository.
- **input-404:** The name of the markdown file used for missing files.
  The corresponding output file will be the same, with the extension replaced with `html`.
  Defaults to `404.md`.
- **site-url:** The url where the book will be hosted. This is required to ensure
  navigation links and script/css imports in the 404 file work correctly, even when accessing
  urls in subdirectories. Defaults to `/`. If `site-url` is set,
  make sure to use document relative links for your assets, meaning they should not start with `/`.
- **cname:** The DNS subdomain or apex domain at which your book will be hosted.
  This string will be written to a file named CNAME in the root of your site, as
  required by GitHub Pages (see [*Managing a custom domain for your GitHub Pages
  site*][custom domain]).

[custom domain]: https://docs.github.com/en/github/working-with-github-pages/managing-a-custom-domain-for-your-github-pages-site

### `[output.html.print]`

The `[output.html.print]` table provides options for controlling the printable output.
By default, mdBook will include an icon on the top right of the book (which looks like <i class="fa fa-print"></i>) that will print the book as a single page.

```toml
[output.html.print]
enable = true    # include support for printable output
page-break = true # insert page-break after each chapter
```

- **enable:** Enable print support. When `false`, all print support will not be
  rendered. Defaults to `true`.
- **page-break:** Insert page breaks between chapters. Defaults to `true`.

### `[output.html.fold]`

The `[output.html.fold]` table provides options for controlling folding of the chapter listing in the navigation sidebar.

```toml
[output.html.fold]
enable = false    # whether or not to enable section folding
level = 0         # the depth to start folding
```

- **enable:** Enable section-folding. When off, all folds are open.
  Defaults to `false`.
- **level:** The higher the more folded regions are open. When level is 0, all
  folds are closed. Defaults to `0`.

### `[output.html.playground]`

The `[output.html.playground]` table provides options for controlling Rust sample code blocks, and their integration with the [Rust Playground].

[Rust Playground]: https://play.rust-lang.org/

```toml
[output.html.playground]
editable = false         # allows editing the source code
copyable = true          # include the copy button for copying code snippets
copy-js = true           # includes the JavaScript for the code editor
line-numbers = false     # displays line numbers for editable code
runnable = true          # displays a run button for rust code
```

- **editable:** Allow editing the source code. Defaults to `false`.
- **copyable:** Display the copy button on code snippets. Defaults to `true`.
- **copy-js:** Copy JavaScript files for the editor to the output directory.
  Defaults to `true`.
- **line-numbers:** Display line numbers on editable sections of code. Requires both `editable` and `copy-js` to be `true`. Defaults to `false`.
- **runnable:** Displays a run button for rust code snippets. Changing this to `false` will disable the run in playground feature globally. Defaults to `true`.

[Ace]: https://ace.c9.io/

### `[output.html.code]`

The `[output.html.code]` table provides options for controlling code blocks.

```toml
[output.html.code]
# A prefix string per language (one or more chars).
# Any line starting with whitespace+prefix is hidden.
hidelines = { python = "~" }
```

- **hidelines:** A table that defines how [hidden code lines](../mdbook.md#hiding-code-lines) work for each language.
  The key is the language and the value is a string that will cause code lines starting with that prefix to be hidden.

### `[output.html.search]`

The `[output.html.search]` table provides options for controlling the built-in text [search].
mdBook must be compiled with the `search` feature enabled (on by default).

[search]: ../../guide/reading.md#search

```toml
[output.html.search]
enable = true            # enables the search feature
limit-results = 30       # maximum number of search results
teaser-word-count = 30   # number of words used for a search result teaser
use-boolean-and = true   # multiple search terms must all match
boost-title = 2          # ranking boost factor for matches in headers
boost-hierarchy = 1      # ranking boost factor for matches in page names
boost-paragraph = 1      # ranking boost factor for matches in text
expand = true            # partial words will match longer terms
heading-split-level = 3  # link results to heading levels
copy-js = true           # include Javascript code for search
```

- **enable:** Enables the search feature. Defaults to `true`.
- **limit-results:** The maximum number of search results. Defaults to `30`.
- **teaser-word-count:** The number of words used for a search result teaser.
  Defaults to `30`.
- **use-boolean-and:** Define the logical link between multiple search words. If
  true, all search words must appear in each result. Defaults to `false`.
- **boost-title:** Boost factor for the search result score if a search word
  appears in the header. Defaults to `2`.
- **boost-hierarchy:** Boost factor for the search result score if a search word
  appears in the hierarchy. The hierarchy contains all titles of the parent
  documents and all parent headings. Defaults to `1`.
- **boost-paragraph:** Boost factor for the search result score if a search word
  appears in the text. Defaults to `1`.
- **expand:** True if search should match longer results e.g. search `micro`
  should match `microwave`. Defaults to `true`.
- **heading-split-level:** Search results will link to a section of the document
  which contains the result. Documents are split into sections by headings this
  level or less. Defaults to `3`. (`### This is a level 3 heading`)
- **copy-js:** Copy JavaScript files for the search implementation to the output
  directory. Defaults to `true`.

### `[output.html.redirect]`

The `[output.html.redirect]` table provides a way to add redirects.
This is useful when you move, rename, or remove a page to ensure that links to the old URL will go to the new location.

```toml
[output.html.redirect]
"/appendices/bibliography.html" = "https://rustc-dev-guide.rust-lang.org/appendix/bibliography.html"
"/other-installation-methods.html" = "../infra/other-installation-methods.html"
```

The table contains key-value pairs where the key is where the redirect file needs to be created, as an absolute path from the build directory, (e.g. `/appendices/bibliography.html`).
The value can be any valid URI the browser should navigate to (e.g. `https://rust-lang.org/`, `/overview.html`, or `../bibliography.html`).

This will generate an HTML page which will automatically redirect to the given location.
Note that the source location does not support `#` anchor redirects.

## Markdown Renderer

The Markdown renderer will run preprocessors and then output the resulting
Markdown. This is mostly useful for debugging preprocessors, especially in
conjunction with `mdbook test` to see the Markdown that `mdbook` is passing
to `rustdoc`.

The Markdown renderer is included with `mdbook` but disabled by default.
Enable it by adding an empty table to your `book.toml` as follows:

```toml
[output.markdown]
```

There are no configuration options for the Markdown renderer at this time;
only whether it is enabled or disabled.

See [the preprocessors documentation](preprocessors.md) for how to
specify which preprocessors should run before the Markdown renderer.
