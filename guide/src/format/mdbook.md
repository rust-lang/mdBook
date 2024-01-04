# mdBook-specific features

## Hiding code lines

There is a feature in mdBook that lets you hide code lines by prepending them with a specific prefix.

For the Rust language, you can use the `#` character as a prefix which will hide lines [like you would with Rustdoc][rustdoc-hide].

[rustdoc-hide]: https://doc.rust-lang.org/stable/rustdoc/write-documentation/documentation-tests.html#hiding-portions-of-the-example

```bash
# fn main() {
    let x = 5;
    let y = 6;

    println!("{}", x + y);
# }
```

Will render as

```rust
# fn main() {
    let x = 5;
    let y = 6;

    println!("{}", x + y);
# }
```

When you tap or hover the mouse over the code block, there will be an eyeball icon (<i class="fa fa-eye"></i>) which will toggle the visibility of the hidden lines.

By default, this only works for code examples that are annotated with `rust`.
However, you can define custom prefixes for other languages by adding a new line-hiding prefix in your `book.toml` with the language name and prefix character(s):

```toml
[output.html.code.hidelines]
python = "~"
```

The prefix will hide any lines that begin with the given prefix. With the python prefix shown above, this:

```bash
~hidden()
nothidden():
~    hidden()
    ~hidden()
    nothidden()
```

will render as

```python
~hidden()
nothidden():
~    hidden()
    ~hidden()
    nothidden()
```

This behavior can be overridden locally with a different prefix. This has the same effect as above:

~~~markdown
```python,hidelines=!!!
!!!hidden()
nothidden():
!!!    hidden()
    !!!hidden()
    nothidden()
```
~~~

## Rust Playground

Rust language code blocks will automatically get a play button (<i class="fa fa-play"></i>) which will execute the code and display the output just below the code block.
This works by sending the code to the [Rust Playground].

```rust
println!("Hello, World!");
```

If there is no `main` function, then the code is automatically wrapped inside one.

If you wish to disable the play button for a code block, you can include the `noplayground` option on the code block like this:

~~~markdown
```rust,noplayground
let mut name = String::new();
std::io::stdin().read_line(&mut name).expect("failed to read line");
println!("Hello {}!", name);
```
~~~

Or, if you wish to disable the play button for all code blocks in your book, you can write the config to the `book.toml` like this.

```toml
[output.html.playground]
runnable = false
```

## Rust code block attributes

Additional attributes can be included in Rust code blocks with comma, space, or tab-separated terms just after the language term. For example:

~~~markdown
```rust,ignore
# This example won't be tested.
panic!("oops!");
```
~~~

These are particularly important when using [`mdbook test`] to test Rust examples.
These use the same attributes as [rustdoc attributes], with a few additions:

* `editable` — Enables the [editor].
* `noplayground` — Removes the play button, but will still be tested.
* `mdbook-runnable` — Forces the play button to be displayed.
  This is intended to be combined with the `ignore` attribute for examples that should not be tested, but you want to allow the reader to run.
* `ignore` — Will not be tested and no play button is shown, but it is still highlighted as Rust syntax.
* `should_panic` — When executed, it should produce a panic.
* `no_run` — The code is compiled when tested, but it is not run.
  The play button is also not shown.
* `compile_fail` — The code should fail to compile.
* `edition2015`, `edition2018`, `edition2021` — Forces the use of a specific Rust edition.
  See [`rust.edition`] to set this globally.

[`mdbook test`]: ../cli/test.md
[rustdoc attributes]: https://doc.rust-lang.org/rustdoc/documentation-tests.html#attributes
[editor]: theme/editor.md
[`rust.edition`]: configuration/general.md#rust-options

## Including files

With the following syntax, you can include files into your book:

```hbs
\{{#include file.rs}}
```

The path to the file has to be relative from the current source file.

mdBook will interpret included files as Markdown. Since the include command
is usually used for inserting code snippets and examples, you will often
wrap the command with ```` ``` ```` to display the file contents without
interpreting them.

````hbs
```
\{{#include file.rs}}
```
````

## Including portions of a file
Often you only need a specific part of the file, e.g. relevant lines for an
example. We support four different modes of partial includes:

```hbs
\{{#include file.rs:2}}
\{{#include file.rs::10}}
\{{#include file.rs:2:}}
\{{#include file.rs:2:10}}
```

The first command only includes the second line from file `file.rs`. The second
command includes all lines up to line 10, i.e. the lines from 11 till the end of
the file are omitted. The third command includes all lines from line 2, i.e. the
first line is omitted. The last command includes the excerpt of `file.rs`
consisting of lines 2 to 10.

To avoid breaking your book when modifying included files, you can also
include a specific section using anchors instead of line numbers.
An anchor is a pair of matching lines. The line beginning an anchor must
match the regex `ANCHOR:\s*[\w_-]+` and similarly the ending line must match
the regex `ANCHOR_END:\s*[\w_-]+`. This allows you to put anchors in
any kind of commented line.

Consider the following file to include:
```rs
/* ANCHOR: all */

// ANCHOR: component
struct Paddle {
    hello: f32,
}
// ANCHOR_END: component

////////// ANCHOR: system
impl System for MySystem { ... }
////////// ANCHOR_END: system

/* ANCHOR_END: all */
```

Then in the book, all you have to do is:
````hbs
Here is a component:
```rust,no_run,noplayground
\{{#include file.rs:component}}
```

Here is a system:
```rust,no_run,noplayground
\{{#include file.rs:system}}
```

This is the full file.
```rust,no_run,noplayground
\{{#include file.rs:all}}
```
````

Lines containing anchor patterns inside the included anchor are ignored.

## Including a file but initially hiding all except specified lines

The `rustdoc_include` helper is for including code from external Rust files that contain complete
examples, but only initially showing particular lines specified with line numbers or anchors in the
same way as with `include`.

The lines not in the line number range or between the anchors will still be included, but they will
be prefaced with `#`. This way, a reader can expand the snippet to see the complete example, and
Rustdoc will use the complete example when you run `mdbook test`.

For example, consider a file named `file.rs` that contains this Rust program:

```rust
fn main() {
    let x = add_one(2);
    assert_eq!(x, 3);
}

fn add_one(num: i32) -> i32 {
    num + 1
}
```

We can include a snippet that initially shows only line 2 by using this syntax:

````hbs
To call the `add_one` function, we pass it an `i32` and bind the returned value to `x`:

```rust
\{{#rustdoc_include file.rs:2}}
```
````

This would have the same effect as if we had manually inserted the code and hidden all but line 2
using `#`:

````hbs
To call the `add_one` function, we pass it an `i32` and bind the returned value to `x`:

```rust
# fn main() {
    let x = add_one(2);
#     assert_eq!(x, 3);
# }
#
# fn add_one(num: i32) -> i32 {
#     num + 1
# }
```
````

That is, it looks like this (click the "expand" icon to see the rest of the file):

```rust
# fn main() {
    let x = add_one(2);
#     assert_eq!(x, 3);
# }
#
# fn add_one(num: i32) -> i32 {
#     num + 1
# }
```

## Inserting runnable Rust files

With the following syntax, you can insert runnable Rust files into your book:

```hbs
\{{#playground file.rs}}
```

The path to the Rust file has to be relative from the current source file.

When play is clicked, the code snippet will be sent to the [Rust Playground] to be
compiled and run. The result is sent back and displayed directly underneath the
code.

Here is what a rendered code snippet looks like:

{{#playground example.rs}}

Any additional values passed after the filename will be included as attributes of the code block.
For example `\{{#playground example.rs editable}}` will create the code block like the following:

~~~markdown
```rust,editable
# Contents of example.rs here.
```
~~~

And the `editable` attribute will enable the [editor] as described at [Rust code block attributes](#rust-code-block-attributes).

[Rust Playground]: https://play.rust-lang.org/

## Controlling page \<title\>

A chapter can set a \<title\> that is different from its entry in the table of
contents (sidebar) by including a `\{{#title ...}}` near the top of the page.

```hbs
\{{#title My Title}}
```

## HTML classes provided by mdBook

<img class="right" src="images/rust-logo-blk.svg" alt="The Rust logo">

### `class="left"` and `"right"`

These classes are provided by default, for inline HTML to float images.

```html
<img class="right" src="images/rust-logo-blk.svg" alt="The Rust logo">
```

### `class="hidden"`

HTML tags with class `hidden` will not be shown.

```html
<div class="hidden">This will not be seen.</div>
```

<div class="hidden">This will not be seen.</div>

### `class="warning"`

To make a warning or similar note stand out, wrap it in a warning div.

```html
<div class="warning">

This is a bad thing that you should pay attention to.

Warning blocks should be used sparingly in documentation, to avoid "warning
fatigue," where people are trained to ignore them because they usually don't
matter for what they're doing.

</div>
```

<div class="warning">

This is a bad thing that you should pay attention to.

Warning blocks should be used sparingly in documentation, to avoid "warning
fatigue," where people are trained to ignore them because they usually don't
matter for what they're doing.

</div>
