# Custom Playground

By default, mdBook supports playground for Rust only.
If you want to add playground for another language, you can do it by
overriding theme assets and adding a small initialization script.

In this section, we will try to add a playground for the imaginary
`example` language.

## Add `playground` option to code blocks in the book

`playground` marks a code block as runnable. `editable` makes the block
an ACE editor as well.

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
[output.html]
additional-css = ["theme/css/example.css"]
additional-js  = ["mode-example.js", "example-init.js"]

[output.html.playground]
editable = true
```

## Prepare syntax highlighting

[highlight.js](https://highlightjs.org) is used for non-editable code
blocks, and [Ace](https://ace.c9.io) is used for `editable` blocks.
Prepare a definition of `example` language for each library.

* highlight.js: place a custom bundle at `theme/highlight.js`.
* Ace: place the mode file at the project root as `mode-example.js`.

## Modify `theme/book.js`

Copy the default `book.js` shipped with the installed mdBook to
`theme/book.js`, then apply the patches below.

### Restrict the `.playground` selector to `<pre>`

`book.js` assumes a `<pre class="playground">` wrapper, but for non-Rust
playgrounds mdBook emits the class on the inner `<code>`. Restrict two
selectors so the `book.js` code paths only act on `<pre>`:

```javascript
const playgrounds = Array.from(document.querySelectorAll('pre.playground'));
```

```javascript
Array.from(document.querySelectorAll('pre.playground')).forEach(function(pre_block) {
```

### Strip `language-example` from editable blocks

Add a `classList.remove` for the language class so highlight.js does
not interfere with the ACE editor inside editable blocks:

```javascript
code_nodes
    .filter(function(node) { return node.classList.contains('editable'); })
    .forEach(function(block) {
        block.classList.remove('language-rust');
        block.classList.remove('language-example');
    });
```

## Add `example-init.js`

Create `example-init.js` at the project root. It runs after the default
ACE initializer and finishes wiring up each playground.

```javascript
"use strict";

function wrapInPlaygroundPre(codeEl) {
    const parent = codeEl.parentNode;
    if (parent.tagName === "PRE") {
        parent.classList.add("playground");
        codeEl.classList.remove("playground");
        return parent;
    }
    const wrapper = document.createElement("pre");
    wrapper.className = "playground";
    codeEl.classList.remove("playground");
    parent.replaceChild(wrapper, codeEl);
    wrapper.appendChild(codeEl);
    return wrapper;
}

function ensureResultBlock(preBlock) {
    let next = preBlock.nextElementSibling;
    if (!(next && next.classList.contains("example-result"))) {
        next = document.createElement("pre");
        next.className = "example-result";
        const code = document.createElement("code");
        next.appendChild(code);
        preBlock.insertAdjacentElement("afterend", next);
    }
    return next.querySelector("code");
}

function paintResult(codeEl, text, isError) {
    codeEl.className = isError ? "language-text" : "language-example";
    codeEl.textContent = text;
    if (window.hljs && !isError) {
        const highlight =
            window.hljs.highlightElement || window.hljs.highlightBlock;
        highlight && highlight.call(window.hljs, codeEl);
    }
}

function readSource(codeEl, editor) {
    return editor ? editor.getValue() : codeEl.textContent;
}

function runExample(preBlock, codeEl, editor) {
    const resultCode = ensureResultBlock(preBlock);
    try {
        // Replace with your own runner. The result string is written
        // into resultCode and highlighted via highlight.js.
        const out = example_language_run(readSource(codeEl, editor));
        paintResult(resultCode, out, false);
    } catch (err) {
        paintResult(resultCode, String(err), true);
    }
}

function iconHTML(id, fallback) {
    const tpl = document.getElementById(id);
    return tpl ? tpl.innerHTML : fallback;
}

function addPlaygroundButtons(preBlock, codeEl, editor) {
    if (preBlock.querySelector(":scope > .buttons > .play-button")) return;
    let buttons = preBlock.querySelector(":scope > .buttons");
    if (!buttons) {
        buttons = document.createElement("div");
        buttons.className = "buttons";
        preBlock.insertBefore(buttons, preBlock.firstChild);
    }

    const playBtn = document.createElement("button");
    playBtn.className = "play-button";
    playBtn.type = "button";
    playBtn.title = "Run this code";
    playBtn.innerHTML = iconHTML("fa-play", "&#9658;");
    playBtn.addEventListener("click", () =>
        runExample(preBlock, codeEl, editor));
    buttons.insertBefore(playBtn, buttons.firstChild);

    if (editor) {
        const resetBtn = document.createElement("button");
        resetBtn.className = "reset-button";
        resetBtn.type = "button";
        resetBtn.title = "Undo changes";
        resetBtn.innerHTML = iconHTML("fa-clock-rotate-left", "&#8634;");
        resetBtn.addEventListener("click", () => {
            editor.setValue(editor.originalCode);
            editor.clearSelection();
        });
        buttons.insertBefore(resetBtn, playBtn.nextSibling);
    }
}

function editorFor(codeEl) {
    return (window.editors || []).find(e => e.container === codeEl) || null;
}

window.addEventListener("load", () => {
    document.querySelectorAll(".playground").forEach(codeEl => {
        if (codeEl.tagName === "PRE") return;
        const editor = editorFor(codeEl);
        if (editor) editor.getSession().setMode("ace/mode/example");
        const preBlock = wrapInPlaygroundPre(codeEl);
        addPlaygroundButtons(preBlock, codeEl, editor);
    });
});
```

Replace `example_language_run` with a call to your runtime. Icons are
pulled from `<template id="fa-play">` and `<template id="fa-clock-rotate-left">`
that the default template provides.

## Add `theme/css/example.css`

```css
pre.playground > .buttons {
    visibility: visible;
    opacity: 1;
}

pre.example-result {
    margin-top: 0.25em;
    border-left: 4px solid var(--quote-border);
}

pre.example-result > code {
    white-space: pre-wrap;
}
```

## Directory structure

The final directory structure becomes below:

```
.
├── book.toml
├── example-init.js
├── mode-example.js
├── src
└── theme
    ├── book.js
    ├── highlight.js
    └── css
        └── example.css
```
