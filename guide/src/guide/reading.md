# Reading Books

This chapter gives an introduction on how to interact with a book produced by mdBook.
This assumes you are reading an HTML book.
The options and formatting will be different for other output formats such as PDF.

A book is organized into *chapters*.
Each chapter is a separate page.
Chapters can be nested into a hierarchy of sub-chapters.
Typically, each chapter will be organized into a series of *headings* to subdivide a chapter.

## Navigation

There are several methods for navigating through the chapters of a book.

The **sidebar** on the left provides a list of all chapters.
Clicking on any of the chapter titles will load that page.

The sidebar may not automatically appear if the window is too narrow, particularly on mobile displays.
In that situation, the menu icon (three horizontal bars) at the top-left of the page can be pressed to open and close the sidebar.

The **arrow buttons** at the bottom of the page can be used to navigate to the previous or the next chapter.

The **left and right arrow keys** on the keyboard can be used to navigate to the previous or the next chapter.

## Top menu bar

The menu bar at the top of the page provides some icons for interacting with the book.
The icons displayed will depend on the settings of how the book was generated.

| Icon | Description |
|------|-------------|
| <i class="fa fa-bars"></i> | Opens and closes the chapter listing sidebar. |
| <i class="fa fa-paint-brush"></i> | Opens a picker to choose a different color theme. |
| <i class="fa fa-search"></i> | Opens a search bar for searching within the book. |
| <i class="fa fa-print"></i> | Instructs the web browser to print the entire book. |
| <i class="fa fa-github"></i> | Opens a link to the website that hosts the source code of the book. |
| <i class="fa fa-edit"></i> | Opens a page to directly edit the source of the page you are currently reading. |

Tapping the menu bar will scroll the page to the top.

## Search

Each book has a built-in search system.
Pressing the search icon (<i class="fa fa-search"></i>) in the menu bar, or pressing the `S` key on the keyboard will open an input box for entering search terms.
Typing some terms will show matching chapters and sections in real time.

Clicking any of the results will jump to that section.
The up and down arrow keys can be used to navigate the results, and enter will open the highlighted section.

After loading a search result, the matching search terms will be highlighted in the text.
Clicking a highlighted word or pressing the `Esc` key will remove the highlighting.

## Code blocks

mdBook books are often used for programming projects, and thus support highlighting code blocks and samples.
Code blocks may contain several different icons for interacting with them:

| Icon | Description |
|------|-------------|
| <i class="fa fa-copy"></i> | Copies the code block into your local clipboard, to allow pasting into another application. |
| <i class="fa fa-play"></i> | For Rust code examples, this will execute the sample code and display the compiler output just below the example (see [playground]). |
| <i class="fa fa-eye"></i> | For Rust code examples, this will toggle visibility of "hidden" lines. Sometimes, larger examples will hide lines which are not particularly relevant to what is being illustrated (see [hiding code lines]). |
| <i class="fa fa-history"></i> | For [editable code examples][editor], this will undo any changes you have made. |

Here's an example:

```rust
println!("Hello, World!");
```

[editor]: ../format/theme/editor.md
[playground]: ../format/mdbook.md#rust-playground
[hiding code lines]: ../format/mdbook.md#hiding-code-lines
