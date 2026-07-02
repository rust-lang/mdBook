# Custom Playground

By default, mdBook supports playground for Rust only.
If you want to add playground for another language,
you can do it by overriding script through theme customization.

In this section, we will try to add a playground for the imaginary `example` language.

## Add `playground` option to code blocks in the book

`playground` option is not officially supported.
It is used to mark the code block should be translated to playground code block by JavaScript.

````markdown
```example,playground
Example Language Code
```
````

`editable` option can be used like below:

````markdown
```example,playground,editable
Example Language Code
```
````

## Modify `book.js`

`book.js` can be modified through theme customization.

### Remove the access to play.rust-lang.org

If there are playgrounds in the page, `book.js` will try to fetch crates list from play.rust-lang.org.
So you shoud remove the following codes.

```javascript
    var playgrounds = Array.from(document.querySelectorAll(".playground"));
    if (playgrounds.length > 0) {
        fetch_with_timeout("https://play.rust-lang.org/meta/crates", {
            headers: {
                'Content-Type': "application/json",
            },
            method: 'POST',
            mode: 'cors',
        })
        .then(response => response.json())
        .then(response => {
            // get list of crates available in the rust playground
            let playground_crates = response.crates.map(item => item["id"]);
            playgrounds.forEach(block => handle_crate_list_update(block, playground_crates));
        });
    }
```

### Modify `run_rust_code` function

If a play button is pushed, `run_rust_code` function will be called to get the result of the playground.
So you should modify the function for your language.

```javascript
    function run_rust_code(code_block) {
        var result_block = code_block.querySelector(".result");
        if (!result_block) {
            result_block = document.createElement('code');
            // If the result should be syntax highlighted,
            // `language-bash` should be modified to the appropriate language.
            result_block.className = 'result hljs language-bash';

            code_block.append(result_block);
        }

        let text = playground_text(code_block);

        // Add function to get result of the playground
        result_block.innerText = example_language_run(text);

        // If the result should be syntax highlighted, enable the following code.
        //hljs.highlightBlock(result_block);
    }
```

### Add code for code block transformation

The following code transforms code blocks which have `playground` option to playground code blocks.

```javascript
    // Add <pre class="playground"> to playground codeblock
    Array.from(document.querySelectorAll(".playground")).forEach((element) => {
        let parent = element.parentNode;
        let wrapper = document.createElement('pre');
        wrapper.className = 'playground';
        element.classList.remove('playground');
        // set the wrapper as child (instead of the element)
        parent.replaceChild(wrapper, element);
        // set element as child of wrapper
        wrapper.appendChild(element);
    });
```

This should be executed before other processes of `codeSnippets` function.
For example, the code should be inserted to the following point.

```javascript
    // Insert the above code

    // Syntax highlighting Configuration
    hljs.configure({
        tabReplace: '    ', // 4 spaces
        languages: [],      // Languages used for auto-detection
    });

    let code_nodes = Array
        .from(document.querySelectorAll('code'))
        // Don't highlight `inline code` blocks in headers.
        .filter(function (node) {return !node.parentElement.classList.contains("header"); });
```

### Tweak for editable code blocks

If the code block is editable, the syntax highlight is executed by editor.
So the normal highlight mechanism should be disabled to avoid conflict.

This is achieved by the following code.

```javascript
        // language-rust class needs to be removed for editable
        // blocks or highlightjs will capture events
        code_nodes
            .filter(function (node) {return node.classList.contains("editable"); })
            .forEach(function (block) { block.classList.remove('language-rust'); });
```

The language name shoud be changed to `language-example`.

```javascript
        // language-rust class needs to be removed for editable
        // blocks or highlightjs will capture events
        code_nodes
            .filter(function (node) {return node.classList.contains("editable"); })
            .forEach(function (block) { block.classList.remove('language-example'); });
```

## Prepare syntax highlighting

[highlight.js](https://highlightjs.org) and [Ace](https://ace.c9.io) are used for syntax highlighting.
If the code block is `editable`, `Ace` is used, and if not `highlight.js` is used.
So a syntax definition of `example` language should be prepared for each library.

You can get the following JavaScript codes by following each library's document.

* highlight.js: `build/highlight.min.js`
* Ace: `build/src-min-noconflict/mode-example.js`

`highlight.js` can be overridden by theme.
So the `build/highlight.min.js` should be moved to `theme/highlight.js` in the project.
`mode-example.js` shoud be moved to the project root because it is not overridable.

## Modify `editor.js`

The language configuration of Ace editor is set at `editor.js`.
This script can't be customized through theme.
So you should copy `editor.js` from the generated book directory to the project root.
The copied file should be modified like below:

```javascript
        //editor.getSession().setMode("ace/mode/rust");
        editor.getSession().setMode("ace/mode/example");
```

## Set additional-js

You should add `editor.js` and `mode-example.js` to `additional-js` field of `book.toml`.

```toml
[output.html]
additional-js = [
  "mode-example.js",
  "editor.js",
]
```

## Directory structure

The final directory structure becomes below:

```
.
|-- book.toml
|-- editor.js
|-- mode-example.js
|-- src
`-- theme
    |-- book.js
    `-- highlight.js
```
