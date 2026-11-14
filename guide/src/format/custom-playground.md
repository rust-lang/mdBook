# Custom Playground

By default, mdBook supports playground for Rust only.
If you want to add playground for another language, you can do it by
registering the language with the syntax highlighters and adding a small
initialization script.

In this section, we will try to add a playground for the imaginary
`example` language.

## Add `playground` option to code blocks in the book

`playground` marks a code block as runnable. `editable` makes the block
an Ace editor as well.

````markdown
```example,playground
example language code
```
````

````markdown
```example,playground,editable
example language code
```
````

## Configure `book.toml`

```toml
{{#include ../../../tests/gui/books/custom-playground/book.toml}}
```

## Prepare syntax highlighting

[highlight.js](https://highlightjs.org) is used for non-editable code
blocks, and [Ace](https://ace.c9.io) is used for `editable` blocks.
Prepare a definition of `example` language for each library.

### highlight.js

Put `hljs-example.js` that registers the language with `hljs.registerLanguage`:

```javascript
{{#include ../../../tests/gui/books/custom-playground/hljs-example.js}}
```

### Ace

Put `mode-example.js` that defines the mode with `ace.define`:

```javascript
{{#include ../../../tests/gui/books/custom-playground/mode-example.js}}
```

## Add `example-init.js`

mdBook adds the run button only to Rust playgrounds. `example-init.js`
turns the `example` code blocks into playgrounds and adds the buttons.

```javascript
{{#include ../../../tests/gui/books/custom-playground/example-init.js}}
```

Replace `example_language_run` with a call to your runtime. The icons are
taken from `<template id="fa-play">` and `<template id="fa-clock-rotate-left">`
in the default template.

## Add `example.css`

```css
{{#include ../../../tests/gui/books/custom-playground/example.css}}
```

The first rule keeps the buttons visible without hovering. It is optional.

## Directory structure

The final directory structure becomes below:

```text
.
├── book.toml
├── example-init.js
├── example.css
├── hljs-example.js
├── mode-example.js
└── src
```
